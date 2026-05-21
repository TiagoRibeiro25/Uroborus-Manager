use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tauri::{AppHandle, Manager};
use walkdir::WalkDir;

const DEFAULT_GAME_PATH: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Resident Evil 5";
const STATE_FILE_NAME: &str = "state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ModEntry {
    id: String,
    name: String,
    version: Option<String>,
    author: Option<String>,
    description: Option<String>,
    enabled: bool,
    files: Vec<String>,
    replaces: Vec<String>,
    screenshot: Option<String>,
    source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct AppState {
    game_path: Option<String>,
    mods_path: Option<String>,
    backup_path: Option<String>,
    mods: Vec<ModEntry>,
    #[serde(default)]
    pending_removals: Vec<RemovedMod>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UiState {
    game_path: Option<String>,
    game_path_valid: bool,
    mods_path: String,
    backup_path: String,
    mods: Vec<ModEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApplyReport {
    applied_mods: usize,
    files_written: usize,
    files_removed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RemovedMod {
    id: String,
    files: Vec<String>,
}

#[tauri::command]
fn get_state(app: AppHandle) -> Result<UiState, String> {
    let mut state = load_state(&app)?;
    ensure_default_paths(&app, &mut state)?;
    save_state(&app, &state)?;
    Ok(build_ui_state(&app, &state)?)
}

#[tauri::command]
fn set_game_path(app: AppHandle, path: String) -> Result<UiState, String> {
    let mut state = load_state(&app)?;
    let candidate = PathBuf::from(path.clone());
    if !is_valid_game_path(&candidate) {
        return Err("Selected folder does not look like Resident Evil 5.".to_string());
    }
    state.game_path = Some(path);
    refresh_replacements(&mut state)?;
    save_state(&app, &state)?;
    Ok(build_ui_state(&app, &state)?)
}

#[tauri::command]
fn set_mods_path(app: AppHandle, path: String) -> Result<UiState, String> {
    let mut state = load_state(&app)?;
    let new_path = PathBuf::from(path.clone());
    if !new_path.exists() {
        fs::create_dir_all(&new_path).map_err(|err| err.to_string())?;
    }
    let current_mods_dir = resolve_mods_dir(&app, &state)?;
    if current_mods_dir != new_path {
        move_dir_contents(&current_mods_dir, &new_path)?;
    }
    state.mods_path = Some(path);
    save_state(&app, &state)?;
    Ok(build_ui_state(&app, &state)?)
}

#[tauri::command]
fn set_backup_path(app: AppHandle, path: String) -> Result<UiState, String> {
    let mut state = load_state(&app)?;
    let new_path = PathBuf::from(path.clone());
    if !new_path.exists() {
        fs::create_dir_all(&new_path).map_err(|err| err.to_string())?;
    }
    let current_backup_dir = resolve_backup_dir(&app, &state)?;
    if current_backup_dir != new_path {
        move_dir_contents(&current_backup_dir, &new_path)?;
    }
    state.backup_path = Some(path);
    save_state(&app, &state)?;
    Ok(build_ui_state(&app, &state)?)
}

#[tauri::command]
fn import_mod_archive(app: AppHandle, archive_path: String) -> Result<ModEntry, String> {
    let mut state = load_state(&app)?;
    ensure_default_paths(&app, &mut state)?;

    let archive = PathBuf::from(&archive_path);
    if !archive.exists() {
        return Err("Archive does not exist.".to_string());
    }

    let mods_dir = resolve_mods_dir(&app, &state)?;
    let temp_dir = resolve_temp_dir(&app)?;
    let mod_id = uuid::Uuid::new_v4().to_string();
    let extract_dir = temp_dir.join(&mod_id);
    fs::create_dir_all(&extract_dir).map_err(|err| err.to_string())?;

    extract_archive(&app, &archive, &extract_dir)?;

    let native_dir = find_native_dir(&extract_dir)?;
    let mod_root = native_dir
        .parent()
        .ok_or_else(|| "Could not resolve mod root.".to_string())?
        .to_path_buf();

    let final_mod_dir = mods_dir.join(&mod_id);
    if final_mod_dir.exists() {
        fs::remove_dir_all(&final_mod_dir).map_err(|err| err.to_string())?;
    }
    fs::create_dir_all(&final_mod_dir).map_err(|err| err.to_string())?;
    move_dir_contents(&mod_root, &final_mod_dir)?;
    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir).ok();
    }

    let modinfo_path = final_mod_dir.join("modinfo.ini");
    let (name, author, version, description) = parse_modinfo(&modinfo_path, &archive);

    let screenshot_path = final_mod_dir.join("screenshot.png");
    let screenshot = if screenshot_path.exists() {
        Some(screenshot_path.to_string_lossy().to_string())
    } else {
        None
    };

    let files = list_mod_files(&final_mod_dir.join("nativePC_MT"))?;
    let replaces = compute_replacements(state.game_path.as_deref(), &files)?;

    let mod_entry = ModEntry {
        id: mod_id,
        name,
        version,
        author,
        description,
        enabled: false,
        files,
        replaces,
        screenshot,
        source: Some(archive_path),
    };

    state.mods.push(mod_entry.clone());
    save_state(&app, &state)?;
    Ok(mod_entry)
}

#[tauri::command]
fn set_mod_enabled(app: AppHandle, id: String, enabled: bool) -> Result<UiState, String> {
    let mut state = load_state(&app)?;
    if let Some(mod_entry) = state.mods.iter_mut().find(|entry| entry.id == id) {
        mod_entry.enabled = enabled;
    }
    save_state(&app, &state)?;
    Ok(build_ui_state(&app, &state)?)
}

#[tauri::command]
fn remove_mod(app: AppHandle, id: String) -> Result<UiState, String> {
    let mut state = load_state(&app)?;
    let mods_dir = resolve_mods_dir(&app, &state)?;
    if let Some(index) = state.mods.iter().position(|entry| entry.id == id) {
        let removed = state.mods.remove(index);
        state.pending_removals.push(RemovedMod {
            id: removed.id,
            files: removed.files,
        });
    }
    let mod_dir = mods_dir.join(&id);
    if mod_dir.exists() {
        fs::remove_dir_all(&mod_dir).map_err(|err| err.to_string())?;
    }
    save_state(&app, &state)?;
    Ok(build_ui_state(&app, &state)?)
}

#[tauri::command]
fn apply_mods(app: AppHandle) -> Result<ApplyReport, String> {
    let mut state = load_state(&app)?;
    let game_path = state
        .game_path
        .clone()
        .ok_or_else(|| "Game path is not set.".to_string())?;
    let game_dir = PathBuf::from(game_path);
    if !is_valid_game_path(&game_dir) {
        return Err("Game path is not valid.".to_string());
    }

    let mods_dir = resolve_mods_dir(&app, &state)?;
    let backup_dir = resolve_backup_dir(&app, &state)?;
    let originals_dir = backup_dir.join("originals");
    fs::create_dir_all(&originals_dir).map_err(|err| err.to_string())?;

    let enabled_mods: Vec<ModEntry> = state
        .mods
        .iter()
        .filter(|entry| entry.enabled)
        .cloned()
        .collect();

    let mut all_mod_files: HashSet<String> = HashSet::new();
    let mut enabled_files: HashSet<String> = HashSet::new();
    for entry in &state.mods {
        for file in &entry.files {
            all_mod_files.insert(file.clone());
        }
        if entry.enabled {
            for file in &entry.files {
                enabled_files.insert(file.clone());
            }
        }
    }

    for removed in &state.pending_removals {
        for file in &removed.files {
            all_mod_files.insert(file.clone());
        }
    }

    let mut files_removed = 0usize;
    for rel in &all_mod_files {
        let game_file = game_dir.join("nativePC_MT").join(rel);
        let backup_file = originals_dir.join("nativePC_MT").join(rel);
        if backup_file.exists() {
            copy_file(&backup_file, &game_file)?;
        } else if !enabled_files.contains(rel) && game_file.exists() {
            fs::remove_file(&game_file).map_err(|err| err.to_string())?;
            files_removed += 1;
        }
    }

    let mut files_written = 0usize;
    for entry in &enabled_mods {
        let mod_root = mods_dir.join(&entry.id).join("nativePC_MT");
        for rel in &entry.files {
            let mod_file = mod_root.join(rel);
            let game_file = game_dir.join("nativePC_MT").join(rel);
            let backup_file = originals_dir.join("nativePC_MT").join(rel);

            if !backup_file.exists() && game_file.exists() {
                copy_file(&game_file, &backup_file)?;
            }

            copy_file(&mod_file, &game_file)?;
            files_written += 1;
        }
    }

    state.pending_removals.clear();
    save_state(&app, &state)?;

    Ok(ApplyReport {
        applied_mods: enabled_mods.len(),
        files_written,
        files_removed,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_state,
            set_game_path,
            set_mods_path,
            set_backup_path,
            import_mod_archive,
            set_mod_enabled,
            remove_mod,
            apply_mods,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_ui_state(app: &AppHandle, state: &AppState) -> Result<UiState, String> {
    let mods_dir = resolve_mods_dir(app, state)?;
    let backup_dir = resolve_backup_dir(app, state)?;
    let game_path_valid = state
        .game_path
        .as_ref()
        .map(|path| is_valid_game_path(Path::new(path)))
        .unwrap_or(false);
    Ok(UiState {
        game_path: state.game_path.clone(),
        game_path_valid,
        mods_path: mods_dir.to_string_lossy().to_string(),
        backup_path: backup_dir.to_string_lossy().to_string(),
        mods: state.mods.clone(),
    })
}

fn ensure_default_paths(app: &AppHandle, state: &mut AppState) -> Result<(), String> {
    if state.game_path.is_none() {
        let default_path = PathBuf::from(DEFAULT_GAME_PATH);
        if is_valid_game_path(&default_path) {
            state.game_path = Some(DEFAULT_GAME_PATH.to_string());
        }
    }

    let mods_dir = resolve_mods_dir(app, state)?;
    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir).map_err(|err| err.to_string())?;
    }

    Ok(())
}

fn resolve_app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|err| err.to_string())
}

fn resolve_state_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(resolve_app_data_dir(app)?.join(STATE_FILE_NAME))
}

fn resolve_mods_dir(app: &AppHandle, state: &AppState) -> Result<PathBuf, String> {
    if let Some(path) = &state.mods_path {
        Ok(PathBuf::from(path))
    } else {
        Ok(resolve_app_data_dir(app)?.join("mods"))
    }
}

fn resolve_backup_dir(app: &AppHandle, state: &AppState) -> Result<PathBuf, String> {
    if let Some(path) = &state.backup_path {
        Ok(PathBuf::from(path))
    } else {
        Ok(resolve_app_data_dir(app)?.join("backups"))
    }
}

fn resolve_temp_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let temp_dir = resolve_app_data_dir(app)?.join("temp");
    if !temp_dir.exists() {
        fs::create_dir_all(&temp_dir).map_err(|err| err.to_string())?;
    }
    Ok(temp_dir)
}

fn load_state(app: &AppHandle) -> Result<AppState, String> {
    let path = resolve_state_path(app)?;
    if !path.exists() {
        return Ok(AppState::default());
    }
    let contents = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    serde_json::from_str(&contents).map_err(|err| err.to_string())
}

fn save_state(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let path = resolve_state_path(app)?;
    let payload = serde_json::to_string_pretty(state).map_err(|err| err.to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, payload).map_err(|err| err.to_string())
}

fn is_valid_game_path(path: &Path) -> bool {
    path.join("re5dx9.exe").exists() && path.join("nativePC_MT").is_dir()
}

fn find_7z_path(app: &AppHandle) -> Result<PathBuf, String> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let candidate = resource_dir.join("bin").join("7z.exe");
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd.join("src-tauri").join("bin").join("7z.exe");
        if candidate.exists() {
            return Ok(candidate);
        }
        let candidate = cwd.join("bin").join("7z.exe");
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("7z.exe was not found. Place it in src-tauri/bin/7z.exe".to_string())
}

fn extract_archive(app: &AppHandle, archive: &Path, output_dir: &Path) -> Result<(), String> {
    let seven_zip = find_7z_path(app)?;
    let output = Command::new(seven_zip)
        .arg("x")
        .arg("-y")
        .arg(format!("-o{}", output_dir.to_string_lossy()))
        .arg(archive.as_os_str())
        .output()
        .map_err(|err| err.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("7z failed: {}", stderr.trim()));
    }

    Ok(())
}

fn find_native_dir(root: &Path) -> Result<PathBuf, String> {
    let mut candidates = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_dir()
            && entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("nativePC_MT")
        {
            candidates.push(entry.path().to_path_buf());
        }
    }

    match candidates.len() {
        0 => Err("nativePC_MT folder not found in the archive.".to_string()),
        1 => Ok(candidates.remove(0)),
        _ => Err("Multiple nativePC_MT folders found in the archive.".to_string()),
    }
}

fn parse_modinfo(path: &Path, archive: &Path) -> (String, Option<String>, Option<String>, Option<String>) {
    let fallback_name = archive
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown Mod".to_string());

    if !path.exists() {
        return (fallback_name, None, None, None);
    }

    let contents = fs::read_to_string(path).unwrap_or_default();
    let mut fields: HashMap<String, String> = HashMap::new();
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') || line.starts_with('[') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            fields.insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    let name = fields
        .get("name")
        .cloned()
        .unwrap_or_else(|| fallback_name.clone());
    let author = fields.get("author").cloned();
    let version = fields.get("version").cloned();
    let description = fields.get("description").cloned();

    (name, author, version, description)
}

fn list_mod_files(native_dir: &Path) -> Result<Vec<String>, String> {
    if !native_dir.exists() {
        return Err("nativePC_MT folder is missing after extraction.".to_string());
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(native_dir).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let rel = entry
                .path()
                .strip_prefix(native_dir)
                .map_err(|err| err.to_string())?;
            files.push(normalize_path(rel));
        }
    }
    Ok(files)
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn compute_replacements(game_path: Option<&str>, files: &[String]) -> Result<Vec<String>, String> {
    let Some(path) = game_path else {
        return Ok(Vec::new());
    };

    let game_path = PathBuf::from(path);
    if !is_valid_game_path(&game_path) {
        return Ok(Vec::new());
    }

    let mut replaces = Vec::new();
    for rel in files {
        let candidate = game_path.join("nativePC_MT").join(rel);
        if candidate.exists() {
            replaces.push(rel.clone());
        }
    }
    Ok(replaces)
}

fn refresh_replacements(state: &mut AppState) -> Result<(), String> {
    let game_path = state.game_path.clone();
    for entry in &mut state.mods {
        entry.replaces = compute_replacements(game_path.as_deref(), &entry.files)?;
    }
    Ok(())
}

fn move_dir_contents(from: &Path, to: &Path) -> Result<(), String> {
    if !from.exists() {
        return Ok(());
    }
    if from == to {
        return Ok(());
    }
    fs::create_dir_all(to).map_err(|err| err.to_string())?;
    for entry in fs::read_dir(from).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            copy_file(&path, &target)?;
        }
    }

    for entry in fs::read_dir(from).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).map_err(|err| err.to_string())?;
        } else {
            fs::remove_file(&path).map_err(|err| err.to_string())?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|err| err.to_string())?;
    for entry in WalkDir::new(from).into_iter().filter_map(Result::ok) {
        let rel = entry.path().strip_prefix(from).map_err(|err| err.to_string())?;
        let target = to.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|err| err.to_string())?;
        } else {
            copy_file(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn copy_file(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::copy(from, to).map_err(|err| err.to_string())?;
    Ok(())
}
