//! Filesystem policy shared by the desktop publisher and MCP transport.
//! Unix snapshots and command payloads belong to the current user, even when
//! the system temporary directory is shared and the process umask is permissive.

use std::{
    fs::{self, OpenOptions},
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub fn root() -> PathBuf {
    if let Some(custom) = std::env::var_os("NBCAD_SESSION_DIR")
        .filter(|value| !value.to_string_lossy().trim().is_empty())
    {
        return custom.into();
    }
    #[cfg(unix)]
    let name = format!("nbcad-sessions-{}", current_user());
    #[cfg(not(unix))]
    let name = "nbcad-sessions";
    std::env::temp_dir().join(name)
}

#[cfg(unix)]
fn current_user() -> u32 {
    // geteuid has no preconditions and does not access pointers.
    unsafe { libc::geteuid() }
}

#[cfg(unix)]
fn directory(path: &Path, private: bool) -> io::Result<fs::File> {
    let mut options = OpenOptions::new();
    options.read(true);
    {
        use std::os::unix::{fs::MetadataExt, fs::OpenOptionsExt, fs::PermissionsExt};
        options.custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW);
        let directory = options.open(path)?;
        if directory.metadata()?.uid() != current_user() {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                format!(
                    "session directory is owned by another user: {}",
                    path.display()
                ),
            ));
        }
        if private && directory.metadata()?.permissions().mode() & 0o777 != 0o700 {
            directory.set_permissions(fs::Permissions::from_mode(0o700))?;
        }
        Ok(directory)
    }
}

/// Reject pre-existing foreign-owned or symlink Unix registries before reads.
/// A missing registry means no sessions have been published yet.
pub fn validate_root() -> io::Result<()> {
    #[cfg(unix)]
    match directory(&root(), false) {
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    Ok(())
}

/// Only call this for session transport paths, never a user-selected save path.
pub fn create_dir_all(path: &Path) -> io::Result<()> {
    #[cfg(not(unix))]
    return fs::create_dir_all(path);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        use std::path::Component;
        let root = root();
        let relative = path.strip_prefix(&root).map_err(|_| {
            io::Error::new(
                ErrorKind::InvalidInput,
                "session directory is outside its registry",
            )
        })?;
        if relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "invalid session directory",
            ));
        }
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&root)?;
        let _root = directory(&root, true)?;
        let mut current = root.clone();
        for part in relative.components() {
            current.push(part);
            match fs::DirBuilder::new().mode(0o700).create(&current) {
                Ok(()) => {}
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
            let _directory = directory(&current, true)?;
        }
        Ok(())
    }
}

/// Set the creation mode before opening: chmod after writing leaves a leak.
pub fn private_options() -> OpenOptions {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut options = OpenOptions::new();
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        options
    }
    #[cfg(not(unix))]
    OpenOptions::new()
}

struct StagedFile {
    path: PathBuf,
    file: Option<fs::File>,
}
impl Drop for StagedFile {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
    }
}

/// Publish one complete private payload without truncating another writer's
/// temporary file. Renaming replaces only the destination directory entry.
pub fn atomic_write(path: &Path, content: &[u8]) -> io::Result<()> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("session file has no parent"))?;
    create_dir_all(parent)?;
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::other("session file has no name"))?;
    for _ in 0..32 {
        let mut temporary = std::ffi::OsString::from(".");
        temporary.push(name);
        temporary.push(format!(
            ".{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let temporary = parent.join(temporary);
        let file = match private_options()
            .create_new(true)
            .write(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let mut staged = StagedFile {
            path: temporary,
            file: Some(file),
        };
        let file = staged.file.as_mut().unwrap();
        file.write_all(content)?;
        file.sync_all()?;
        drop(staged.file.take());
        return fs::rename(&staged.path, path);
    }
    Err(io::Error::new(
        ErrorKind::AlreadyExists,
        "could not reserve a session temporary file",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENVIRONMENT: Mutex<()> = Mutex::new(());

    struct TestRoot {
        path: PathBuf,
        previous: Option<std::ffi::OsString>,
    }

    impl TestRoot {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "nbcad-private-sessions-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
            ));
            fs::create_dir(&path).unwrap();
            let previous = std::env::var_os("NBCAD_SESSION_DIR");
            std::env::set_var("NBCAD_SESSION_DIR", &path);
            Self { path, previous }
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => std::env::set_var("NBCAD_SESSION_DIR", value),
                None => std::env::remove_var("NBCAD_SESSION_DIR"),
            }
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn default_and_override_use_the_same_policy() {
        let _lock = ENVIRONMENT.lock().unwrap();
        let fixture = TestRoot::new();
        assert_eq!(root(), fixture.path);
        std::env::remove_var("NBCAD_SESSION_DIR");
        #[cfg(unix)]
        let expected = std::env::temp_dir().join(format!("nbcad-sessions-{}", current_user()));
        #[cfg(not(unix))]
        let expected = std::env::temp_dir().join("nbcad-sessions");
        assert_eq!(root(), expected);
        std::env::set_var("NBCAD_SESSION_DIR", "  ");
        assert_eq!(root(), expected);
    }

    #[test]
    fn concurrent_publication_keeps_whole_payloads_and_cleans_stages() {
        let _lock = ENVIRONMENT.lock().unwrap();
        let fixture = TestRoot::new();
        let target = fixture.path.join("document/model.json");
        let writers: Vec<_> = (0..8)
            .map(|index| {
                let target = target.clone();
                std::thread::spawn(move || atomic_write(&target, &vec![index; 8192]).unwrap())
            })
            .collect();
        for writer in writers {
            writer.join().unwrap();
        }
        let payload = fs::read(&target).unwrap();
        assert_eq!(payload.len(), 8192);
        assert!(payload.iter().all(|byte| *byte == payload[0]));
        assert_eq!(fs::read_dir(target.parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn failed_replacement_preserves_destination_and_cleans_stage() {
        let _lock = ENVIRONMENT.lock().unwrap();
        let fixture = TestRoot::new();
        let target = fixture.path.join("model.json");
        fs::create_dir(&target).unwrap();
        assert!(atomic_write(&target, b"cannot replace directory").is_err());
        assert!(target.is_dir());
        assert_eq!(fs::read_dir(&fixture.path).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn snapshots_and_existing_directories_are_private() {
        use std::os::unix::fs::PermissionsExt;
        let _lock = ENVIRONMENT.lock().unwrap();
        let fixture = TestRoot::new();
        fs::set_permissions(&fixture.path, fs::Permissions::from_mode(0o755)).unwrap();
        let target = fixture.path.join("document/inbox/1.json");
        atomic_write(&target, b"private CAD payload").unwrap();
        for directory in [
            &fixture.path,
            &fixture.path.join("document"),
            &fixture.path.join("document/inbox"),
        ] {
            assert_eq!(
                fs::metadata(directory).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        assert_eq!(
            fs::metadata(target).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_root_and_child_cannot_redirect_or_change_permissions() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let _lock = ENVIRONMENT.lock().unwrap();
        let fixture = TestRoot::new();
        let destination = fixture.path.join("destination");
        fs::create_dir(&destination).unwrap();
        fs::set_permissions(&destination, fs::Permissions::from_mode(0o755)).unwrap();
        let link = fixture.path.join("link");
        symlink(&destination, &link).unwrap();
        std::env::set_var("NBCAD_SESSION_DIR", &link);
        assert!(validate_root().is_err());
        assert!(atomic_write(&link.join("model.json"), b"private").is_err());
        std::env::set_var("NBCAD_SESSION_DIR", &fixture.path);
        assert!(atomic_write(&link.join("model.json"), b"private").is_err());
        assert!(!destination.join("model.json").exists());
        assert_eq!(
            fs::metadata(destination).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }

    #[cfg(unix)]
    #[test]
    fn traversal_cannot_write_outside_registry() {
        let _lock = ENVIRONMENT.lock().unwrap();
        let fixture = TestRoot::new();
        let error =
            atomic_write(&fixture.path.join("../escaped/model.json"), b"private").unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
    }

    #[cfg(unix)]
    #[test]
    fn foreign_root_is_rejected_without_chmod() {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let _lock = ENVIRONMENT.lock().unwrap();
        let fixture = TestRoot::new();
        let foreign = if current_user() == 0 {
            use std::os::unix::ffi::OsStrExt;
            let path = fixture.path.join("foreign");
            fs::create_dir(&path).unwrap();
            let c_path = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
            // This branch only runs as root and changes an owned test directory.
            assert_eq!(unsafe { libc::chown(c_path.as_ptr(), 1, !0) }, 0);
            path
        } else {
            PathBuf::from("/tmp")
        };
        assert_ne!(fs::metadata(&foreign).unwrap().uid(), current_user());
        let before = fs::metadata(&foreign).unwrap().permissions().mode();
        std::env::set_var("NBCAD_SESSION_DIR", &foreign);
        assert_eq!(
            validate_root().unwrap_err().kind(),
            ErrorKind::PermissionDenied
        );
        assert_eq!(
            create_dir_all(&foreign).unwrap_err().kind(),
            ErrorKind::PermissionDenied
        );
        assert_eq!(fs::metadata(&foreign).unwrap().permissions().mode(), before);
    }
}
