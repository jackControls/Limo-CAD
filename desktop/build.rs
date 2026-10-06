fn main() {
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=windows.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        // A library crate's link arguments do not propagate to this executable.
        // Keep copied release binaries able to locate their app's OCCT/TBB bundle.
        println!("cargo:rustc-link-arg-bins=-Wl,-rpath,@executable_path/../Frameworks");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("icons/icon.ico")
            .set("ProductName", "Limo CAD")
            .set("FileDescription", "Limo CAD")
            .set("OriginalFilename", "Limo-CAD.exe")
            .set_manifest_file("windows.manifest")
            .compile()
            .expect("compile native Windows icon, version and DPI manifest");
    }
}
