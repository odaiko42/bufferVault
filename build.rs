// BufferVault - Script de build
// Compile le fichier de ressources (.rc) pour embarquer l'icone dans le binaire

fn main() {
    // Recompiler si le fichier .rc ou .ico change
    println!("cargo:rerun-if-changed=resources/app.rc");
    println!("cargo:rerun-if-changed=resources/app.ico");

    // Trouver rc.exe dans le Windows SDK
    let rc_exe = find_rc_exe().expect("rc.exe not found in Windows SDK");

    // Compiler le .rc en .res
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let res_path = format!("{}\\app.res", out_dir);

    let status = std::process::Command::new(&rc_exe)
        .args(&["/nologo", "/fo", &res_path, "resources\\app.rc"])
        .status()
        .expect("Failed to run rc.exe");

    if !status.success() {
        panic!("rc.exe failed with status: {}", status);
    }

    // Lier le .res au binaire
    println!("cargo:rustc-link-arg={}", res_path);
}

/// Cherche rc.exe dans le Windows 10 SDK.
fn find_rc_exe() -> Option<String> {
    let sdk_root = "C:\\Program Files (x86)\\Windows Kits\\10\\bin";
    let sdk_path = std::path::Path::new(sdk_root);
    if !sdk_path.exists() {
        return None;
    }

    // Lister les versions du SDK et prendre la plus recente
    let mut versions: Vec<_> = std::fs::read_dir(sdk_path)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("10."))
        .map(|e| e.path())
        .collect();

    versions.sort();

    // Chercher rc.exe dans la version la plus recente, architecture x64 puis x86
    for version_dir in versions.iter().rev() {
        for arch in &["x64", "x86"] {
            let rc_path = version_dir.join(arch).join("rc.exe");
            if rc_path.exists() {
                return Some(rc_path.to_string_lossy().into_owned());
            }
        }
    }

    None
}
