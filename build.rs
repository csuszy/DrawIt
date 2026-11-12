fn main() {
    println!("cargo:rerun-if-changed=DrawIt_logo.ico");
    println!("cargo:rerun-if-changed=app.ico");
    println!("cargo:rerun-if-changed=app.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("DrawIt_logo.ico");
        res.set_manifest_file("app.manifest");
        if let Ok(()) = res.compile() {
            if let Ok(out_dir) = std::env::var("OUT_DIR") {
                let obj = std::path::Path::new(&out_dir).join("resource.o");
                if obj.exists() {
                    println!("cargo:rustc-link-arg={}", obj.display());
                }
            }
        }
    }
}
