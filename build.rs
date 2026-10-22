fn main() {
    if std::env::var_os("CARGO_FEATURE_GUI").is_some() {
        println!("cargo:rerun-if-changed=packaging/resources.xml");
        println!("cargo:rerun-if-changed=logo/NexuTerm.png");
        println!("cargo:rerun-if-changed=logo/NexuTerm-toolbar.png");
        let target = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap())
            .join("nexuterm.gresource");
        let status = std::process::Command::new("glib-compile-resources")
            .arg("packaging/resources.xml")
            .arg("--sourcedir=.")
            .arg("--target")
            .arg(target)
            .status()
            .expect("Instala glib-compile-resources (libglib2.0-dev-bin)");
        assert!(status.success(), "No se pudieron compilar los recursos");
    }
    println!("cargo:rerun-if-changed=src/vnc_bridge.c");
    let lib = pkg_config::Config::new()
        .probe("libvncclient")
        .expect("Instala libvncserver-dev");
    let mut build = cc::Build::new();
    build.file("src/vnc_bridge.c");
    for path in lib.include_paths {
        build.include(path);
    }
    build.compile("nexuterm_vnc");
}
