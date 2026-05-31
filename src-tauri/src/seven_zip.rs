use std::path::PathBuf;
use tauri::AppHandle;

#[cfg(windows)]
const EMBEDDED_7Z: &[u8] = include_bytes!("../bin/7z.exe");

/// Resolves a path to 7z.exe, extracting the copy embedded in the binary on first use.
pub fn resolve_7z_executable(app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        resolve_embedded_7z(app)
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err("Archive extraction is only supported on Windows.".to_string())
    }
}

#[cfg(windows)]
fn resolve_embedded_7z(app: &AppHandle) -> Result<PathBuf, String> {
    use std::fs;
    let tools_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| err.to_string())?
        .join("tools");
    let seven_zip = tools_dir.join("7z.exe");
    let expected_len = EMBEDDED_7Z.len() as u64;

    let needs_write = match fs::metadata(&seven_zip) {
        Ok(meta) => meta.len() != expected_len,
        Err(_) => true,
    };

    if needs_write {
        fs::create_dir_all(&tools_dir).map_err(|err| err.to_string())?;
        fs::write(&seven_zip, EMBEDDED_7Z).map_err(|err| err.to_string())?;
    }

    Ok(seven_zip)
}
