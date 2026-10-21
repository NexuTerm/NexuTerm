fn main() {
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
