//! Native OS recipe delivery. Opening a URL only queues installed source;
//! draft/document guards remain owned by the existing Scripts controller.
#[cfg(all(target_os = "linux", not(debug_assertions)))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(all(target_os = "windows", not(debug_assertions)))]
mod windows;

pub(crate) fn install(app: &mut bevy::prelude::App) {
    #[cfg(target_os = "macos")]
    macos::install(app);
    #[cfg(all(target_os = "linux", not(debug_assertions)))]
    if let Err(error) = std::thread::Builder::new()
        .name("cad-recipe-registration".into())
        .spawn(|| {
            if let Err(error) = linux::register() {
                eprintln!("Could not register recipe links: {error}");
            }
        })
    {
        eprintln!("Could not start recipe link registration: {error}");
    }
    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    if let Err(error) = windows::register() {
        eprintln!("Could not register recipe links: {error}");
    }
    #[cfg(not(target_os = "macos"))]
    let _ = app; // Windows/Linux URL launches use the validated argv entry point.
}
