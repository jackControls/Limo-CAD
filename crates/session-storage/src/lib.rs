//! Filesystem policy shared by the desktop publisher and MCP transport.
//! Unix snapshots and command payloads belong to the current user, even when
//! the system temporary directory is shared and the process umask is permissive.

use std::{
    fs::{self, OpenOptions},
    io::{self, ErrorKind, Read, Write},
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
        let metadata = directory.metadata()?;
        let mode = metadata.permissions().mode();
        if path.canonicalize()?.parent().is_none() || mode & 0o1000 != 0 {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "a filesystem root or sticky directory cannot be a session registry",
            ));
        }
        if metadata.uid() != current_user() {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                format!(
                    "session directory is owned by another user: {}",
                    path.display()
                ),
            ));
        }
        if private && mode & 0o077 != 0 {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "session registry must be private; set NBCAD_SESSION_DIR to a dedicated 0700 directory owned by this user",
            ));
        }
        if mode & 0o022 != 0 {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "session directory is writable by another user",
            ));
        }
        Ok(directory)
    }
}

#[cfg(unix)]
fn readable_directory(root: &Path, path: &Path) -> io::Result<()> {
    use std::path::Component;
    let relative = path
        .strip_prefix(root)
        .ok()
        .filter(|relative| {
            relative
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        })
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid session directory"))?;
    let _root = directory(root, true)?;
    let mut current = root.to_path_buf();
    for part in relative.components() {
        current.push(part);
        let _directory = directory(&current, false)?;
    }
    Ok(())
}

/// Discovery must validate every directory between the registry and leases.
pub fn read_dir(path: impl AsRef<Path>) -> io::Result<fs::ReadDir> {
    read_dir_from(&root(), path.as_ref())
}

/// The same discovery policy with an explicit registry for bounded transports.
pub fn read_dir_from(root: &Path, path: &Path) -> io::Result<fs::ReadDir> {
    #[cfg(unix)]
    readable_directory(root, path)?;
    #[cfg(not(unix))]
    let _ = root;
    fs::read_dir(path)
}

/// Read a regular, owned payload inside a private registry without following
/// child/file symlinks or blocking indefinitely while opening a Unix FIFO.
pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    let path = path.as_ref();
    #[cfg(unix)]
    readable_directory(
        &root(),
        path.parent()
            .ok_or_else(|| io::Error::other("session file has no parent"))?,
    )?;
    let mut options = private_options();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = options.read(true).open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "session payload is not a regular file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.uid() != current_user() || metadata.permissions().mode() & 0o022 != 0 {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "session payload is not owned and protected",
            ));
        }
    }
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Reject pre-existing foreign-owned or symlink Unix registries before reads.
/// A missing registry means no sessions have been published yet.
pub fn validate_root() -> io::Result<()> {
    #[cfg(unix)]
    match directory(&root(), true) {
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    Ok(())
}

/// Only call this for session transport paths, never a user-selected save path.
pub fn create_dir_all(path: &Path) -> io::Result<()> {
    create_dir_all_from(&root(), path)
}

/// Create private transport paths without altering existing directory modes.
pub fn create_dir_all_from(root: &Path, path: &Path) -> io::Result<()> {
    #[cfg(not(unix))]
    {
        let _ = root;
        fs::create_dir_all(path)
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        use std::path::Component;
        let relative = path.strip_prefix(root).map_err(|_| {
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
            .create(root)?;
        let _root = directory(root, true)?;
        let mut current = root.to_path_buf();
        for part in relative.components() {
            current.push(part);
            match fs::DirBuilder::new().mode(0o700).create(&current) {
                Ok(()) => {}
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
            let _directory = directory(&current, false)?;
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
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
            }
            #[cfg(not(unix))]
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
    fn new_snapshots_are_private_and_public_roots_are_rejected() {
        use std::os::unix::fs::PermissionsExt;
        let _lock = ENVIRONMENT.lock().unwrap();
        let fixture = TestRoot::new();
        fs::set_permissions(&fixture.path, fs::Permissions::from_mode(0o755)).unwrap();
        let target = fixture.path.join("document/inbox/1.json");
        assert!(atomic_write(&target, b"private CAD payload").is_err());
        assert_eq!(
            fs::metadata(&fixture.path).unwrap().permissions().mode() & 0o777,
            0o755
        );
        fs::set_permissions(&fixture.path, fs::Permissions::from_mode(0o700)).unwrap();
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

    #[cfg(unix)]
    #[test]
    fn global_directory_overrides_are_rejected_without_chmod() {
        use std::os::unix::fs::PermissionsExt;
        let _lock = ENVIRONMENT.lock().unwrap();
        let fixture = TestRoot::new();
        fs::set_permissions(&fixture.path, fs::Permissions::from_mode(0o1777)).unwrap();
        assert!(create_dir_all(&fixture.path).is_err());
        assert_eq!(
            fs::metadata(&fixture.path).unwrap().permissions().mode() & 0o7777,
            0o1777
        );
        // Read-only validation proves canonical root aliases are rejected too.
        std::env::set_var("NBCAD_SESSION_DIR", "/.");
        let error = validate_root().unwrap_err();
        assert!(error.to_string().contains("filesystem root"));
    }

    #[cfg(unix)]
    #[test]
    fn readers_reject_writable_registries_and_child_or_payload_symlinks() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let _lock = ENVIRONMENT.lock().unwrap();
        let fixture = TestRoot::new();
        let document = fixture.path.join("document");
        fs::create_dir(&document).unwrap();
        fs::write(document.join("model.json"), "owned snapshot").unwrap();
        // Descendants may retain legacy modes inside a validated private root.
        assert_eq!(
            read_to_string(document.join("model.json")).unwrap(),
            "owned snapshot"
        );
        fs::set_permissions(&fixture.path, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(read_to_string(document.join("model.json")).is_err());
        assert!(read_dir(&fixture.path).is_err());
        fs::set_permissions(&fixture.path, fs::Permissions::from_mode(0o777)).unwrap();
        assert!(read_to_string(document.join("model.json")).is_err());
        assert!(read_dir(&fixture.path).is_err());
        fs::set_permissions(&fixture.path, fs::Permissions::from_mode(0o700)).unwrap();
        let directory_link = fixture.path.join("_ui");
        symlink(&document, &directory_link).unwrap();
        assert!(read_dir(&directory_link).is_err());
        assert!(read_to_string(directory_link.join("model.json")).is_err());
        symlink(document.join("model.json"), document.join("linked.json")).unwrap();
        assert!(read_to_string(document.join("linked.json")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn privileged_readers_reject_foreign_children_and_payloads() {
        use std::os::unix::ffi::OsStrExt;
        let _lock = ENVIRONMENT.lock().unwrap();
        let fixture = TestRoot::new();
        if current_user() != 0 {
            // The focused Unix CI job repeats this binary under sudo.
            return;
        }
        let document = fixture.path.join("document");
        create_dir_all(&document).unwrap();
        let payload = document.join("model.json");
        atomic_write(&payload, b"owned snapshot").unwrap();
        let change_owner = |path: &Path, owner| {
            let path = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
            // Only paths within the current disposable fixture are changed.
            assert_eq!(unsafe { libc::chown(path.as_ptr(), owner, !0) }, 0);
        };
        change_owner(&payload, 1);
        assert!(read_to_string(&payload).is_err());
        change_owner(&payload, 0);
        change_owner(&document, 1);
        assert!(read_dir(&document).is_err());
        assert!(read_to_string(&payload).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn readers_reject_fifo_payloads_without_waiting_for_a_writer() {
        use std::os::unix::ffi::OsStrExt;
        let _lock = ENVIRONMENT.lock().unwrap();
        let fixture = TestRoot::new();
        let path = fixture.path.join("model.json");
        let c_path = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
        // mkfifo only creates a file in the owned disposable registry.
        assert_eq!(unsafe { libc::mkfifo(c_path.as_ptr(), 0o600) }, 0);
        let started = std::time::Instant::now();
        assert_eq!(
            read_to_string(path).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
    }
}
