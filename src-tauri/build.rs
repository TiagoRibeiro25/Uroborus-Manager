fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let seven_zip = std::path::Path::new("bin/7z.exe");
        if !seven_zip.exists() {
            panic!(
                "Missing src-tauri/bin/7z.exe (required for Windows builds).\n\
                 Place the standalone 7z.exe from 7-Zip Extra in src-tauri/bin/."
            );
        }
        println!("cargo:rerun-if-changed=bin/7z.exe");
    }
    tauri_build::build()
}
