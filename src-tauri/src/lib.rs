use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tauri::{AppHandle, Manager};
use walkdir::WalkDir;

mod seven_zip;

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
struct ApplyConflict {
    file: String,
    mods: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UiState {
    game_path: Option<String>,
    game_path_valid: bool,
    mods_path: String,
    backup_path: String,
    mods: Vec<ModEntry>,
    apply_conflicts: Vec<ApplyConflict>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApplyReport {
    applied_mods: usize,
    files_written: usize,
    files_restored: usize,
    files_removed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RemovedMod {
    id: String,
    files: Vec<String>,
    #[serde(default)]
    replaces: Vec<String>,
}

#[tauri::command]
fn get_state(app: AppHandle) -> Result<UiState, String> {
    let mut state = load_state(&app)?;
    let mut dirty = ensure_default_paths(&app, &mut state)?;
    if reconcile_mods(&app, &mut state)? {
        dirty = true;
    }
    if dirty {
        save_state(&app, &state)?;
    }
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
    let _ = ensure_default_paths(&app, &mut state)?;

    let archive = PathBuf::from(&archive_path);
    if !archive.exists() {
        return Err("Archive does not exist.".to_string());
    }

    let mods_dir = resolve_mods_dir(&app, &state)?;
    let temp_dir = resolve_temp_dir(&app)?;
    let mod_id = uuid::Uuid::new_v4().to_string();
    let extract_dir = temp_dir.join(&mod_id);
    let final_mod_dir = mods_dir.join(&mod_id);

    let import_result = (|| -> Result<ModEntry, String> {
        fs::create_dir_all(&extract_dir).map_err(|err| err.to_string())?;
        extract_archive(&app, &archive, &extract_dir)?;

        let native_dir = find_native_dir(&extract_dir)?;
        let mod_root = native_dir
            .parent()
            .ok_or_else(|| "Could not resolve mod root.".to_string())?
            .to_path_buf();

        if final_mod_dir.exists() {
            fs::remove_dir_all(&final_mod_dir).map_err(|err| err.to_string())?;
        }
        fs::create_dir_all(&final_mod_dir).map_err(|err| err.to_string())?;
        move_dir_contents(&mod_root, &final_mod_dir)?;

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

        Ok(ModEntry {
            id: mod_id.clone(),
            name,
            version,
            author,
            description,
            enabled: false,
            files,
            replaces,
            screenshot,
            source: Some(archive_path),
        })
    })();

    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir).ok();
    }

    let mod_entry = match import_result {
        Ok(entry) => entry,
        Err(err) => {
            if final_mod_dir.exists() {
                fs::remove_dir_all(&final_mod_dir).ok();
            }
            return Err(err);
        }
    };

    state.mods.push(mod_entry.clone());
    save_state(&app, &state)?;
    Ok(mod_entry)
}

#[tauri::command]
fn set_mod_enabled(app: AppHandle, id: String, enabled: bool) -> Result<UiState, String> {
    let mut state = load_state(&app)?;
    let Some(mod_entry) = state.mods.iter_mut().find(|entry| entry.id == id) else {
        return Err(format!("Mod not found: {id}"));
    };
    mod_entry.enabled = enabled;
    save_state(&app, &state)?;
    Ok(build_ui_state(&app, &state)?)
}

#[tauri::command]
fn disable_all_mods(app: AppHandle) -> Result<UiState, String> {
    let mut state = load_state(&app)?;
    for entry in &mut state.mods {
        entry.enabled = false;
    }
    save_state(&app, &state)?;
    Ok(build_ui_state(&app, &state)?)
}

#[tauri::command]
fn reorder_mod(app: AppHandle, id: String, direction: String) -> Result<UiState, String> {
    let mut state = load_state(&app)?;
    let len = state.mods.len();
    let Some(index) = state.mods.iter().position(|entry| entry.id == id) else {
        return Err(format!("Mod not found: {id}"));
    };
    let new_index = match direction.as_str() {
        "up" if index > 0 => index - 1,
        "down" if index + 1 < len => index + 1,
        _ => return Ok(build_ui_state(&app, &state)?),
    };
    state.mods.swap(index, new_index);
    save_state(&app, &state)?;
    Ok(build_ui_state(&app, &state)?)
}

#[tauri::command]
fn remove_mod(app: AppHandle, id: String) -> Result<UiState, String> {
    let mut state = load_state(&app)?;
    let mods_dir = resolve_mods_dir(&app, &state)?;
    if let Some(index) = state.mods.iter().position(|entry| entry.id == id) {
        let removed = state.mods.remove(index);
        // Queue restore on next Apply so files left in the game (e.g. disabled but not re-applied) are cleaned up.
        if !removed.files.is_empty() {
            state.pending_removals.push(RemovedMod {
                id: removed.id,
                files: removed.files,
                replaces: removed.replaces,
            });
        }
    } else {
        return Err(format!("Mod not found: {id}"));
    }
    let mod_dir = mods_dir.join(&id);
    if mod_dir.exists() {
        fs::remove_dir_all(&mod_dir).map_err(|err| err.to_string())?;
    }
    save_state(&app, &state)?;
    Ok(build_ui_state(&app, &state)?)
}

#[tauri::command]
fn launch_game(app: AppHandle) -> Result<(), String> {
    let state = load_state(&app)?;
    let game_path = state
        .game_path
        .ok_or_else(|| "Game path is not set.".to_string())?;
    let game_dir = PathBuf::from(&game_path);
    if !is_valid_game_path(&game_dir) {
        return Err("Game path is not valid.".to_string());
    }
    let exe = game_dir.join("re5dx9.exe");
    std::process::Command::new(&exe)
        .current_dir(&game_dir)
        .spawn()
        .map_err(|err| format!("Failed to launch game: {err}"))?;
    Ok(())
}

#[tauri::command]
fn apply_mods(app: AppHandle) -> Result<ApplyReport, String> {
    let mut state = load_state(&app)?;
    refresh_replacements(&mut state)?;

    let conflicts = collect_apply_conflicts(&state);
    if !conflicts.is_empty() {
        return Err(format_apply_conflict_error(&conflicts));
    }

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
    let backup_native = primary_backup_native(&backup_dir);
    fs::create_dir_all(&backup_native).map_err(|err| err.to_string())?;

    let enabled_mods: Vec<ModEntry> = state
        .mods
        .iter()
        .filter(|entry| entry.enabled)
        .cloned()
        .collect();

    // Files owned by disabled or removed mods only — not every file in the backup folder.
    let mut inactive_files: HashSet<String> = HashSet::new();
    let mut enabled_files: HashSet<String> = HashSet::new();
    for entry in &state.mods {
        if entry.enabled {
            for file in &entry.files {
                enabled_files.insert(file.clone());
            }
        } else {
            for file in &entry.files {
                inactive_files.insert(file.clone());
            }
        }
    }
    for removed in &state.pending_removals {
        for file in &removed.files {
            inactive_files.insert(file.clone());
        }
    }

    let native_pc = game_dir.join("nativePC_MT");

    let mut files_restored = 0usize;
    let mut files_removed = 0usize;
    for rel in &inactive_files {
        // Another enabled mod still uses this path — keep the modded file and its backup.
        if enabled_files.contains(rel) {
            continue;
        }
        let game_file = join_rel(&native_pc, rel)?;
        let expects_vanilla = inactive_file_expects_vanilla(&state, rel);

        if let Some(backup_file) = find_backup_file(&backup_dir, rel) {
            restore_file_from_backup(&backup_file, &game_file)?;
            delete_backup_for_rel(&backup_dir, rel)?;
            files_restored += 1;
        } else if expects_vanilla {
            if game_file.exists() {
                // Backup was already restored (e.g. mod was disabled and applied earlier).
                continue;
            }
            return Err(format!(
                "Missing backup for \"{rel}\". Enable the mod(s) again, click Apply, then disable and Apply once more."
            ));
        } else if game_file.exists() {
            // Mod-added file (was not in the game before the mod).
            fs::remove_file(&game_file).map_err(|err| err.to_string())?;
            files_removed += 1;
        }
    }

    let mut files_written = 0usize;
    let mut written_rels: Vec<String> = Vec::new();
    for entry in &enabled_mods {
        let mod_root = mods_dir.join(&entry.id).join("nativePC_MT");
        for rel in &entry.files {
            let mod_file = join_rel(&mod_root, rel)?;
            if !mod_file.is_file() {
                rollback_writes(&native_pc, &backup_dir, &written_rels)?;
                return Err(format!("Mod file missing: {rel}"));
            }
            let game_file = join_rel(&native_pc, rel)?;
            let backup_file = join_rel(&backup_native, rel)?;

            // Backup vanilla before the first mod overwrites this path.
            if !backup_file.exists() {
                if game_file.exists() {
                    copy_file(&game_file, &backup_file)?;
                } else if entry.replaces.contains(rel) {
                    rollback_writes(&native_pc, &backup_dir, &written_rels)?;
                    return Err(format!(
                        "Game file missing for \"{rel}\". Verify your game path is correct."
                    ));
                }
            }

            if let Err(err) = copy_file(&mod_file, &game_file) {
                rollback_writes(&native_pc, &backup_dir, &written_rels)?;
                return Err(format!(
                    "{err}. Rolled back {count} file(s) written during this apply.",
                    count = written_rels.len()
                ));
            }
            written_rels.push(rel.clone());
            files_written += 1;
        }
    }

    state.pending_removals.clear();
    save_state(&app, &state)?;

    Ok(ApplyReport {
        applied_mods: enabled_mods.len(),
        files_written,
        files_restored,
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
            disable_all_mods,
            reorder_mod,
            remove_mod,
            launch_game,
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
        apply_conflicts: collect_apply_conflicts(state),
    })
}

fn collect_apply_conflicts(state: &AppState) -> Vec<ApplyConflict> {
    let mut file_owners: HashMap<String, Vec<String>> = HashMap::new();
    for entry in state.mods.iter().filter(|entry| entry.enabled) {
        for file in &entry.files {
            file_owners
                .entry(file.clone())
                .or_default()
                .push(entry.name.clone());
        }
    }

    let mut conflicts: Vec<ApplyConflict> = file_owners
        .into_iter()
        .filter(|(_, mods)| mods.len() > 1)
        .map(|(file, mods)| ApplyConflict { file, mods })
        .collect();
    conflicts.sort_by(|left, right| left.file.cmp(&right.file));
    conflicts
}

fn format_apply_conflict_error(conflicts: &[ApplyConflict]) -> String {
    let mut message =
        String::from("Apply blocked: enabled mods share the same game files. Reorder or disable mods to resolve conflicts.\n");
    for conflict in conflicts.iter().take(8) {
        message.push_str(&format!(
            "\n• {} — {}",
            conflict.file,
            conflict.mods.join(" vs ")
        ));
    }
    if conflicts.len() > 8 {
        message.push_str(&format!(
            "\n…and {} more conflict(s).",
            conflicts.len() - 8
        ));
    }
    message
}

fn ensure_default_paths(app: &AppHandle, state: &mut AppState) -> Result<bool, String> {
    let mut changed = false;
    if state.game_path.is_none() {
        let default_path = PathBuf::from(DEFAULT_GAME_PATH);
        if is_valid_game_path(&default_path) {
            state.game_path = Some(DEFAULT_GAME_PATH.to_string());
            changed = true;
        }
    }

    let mods_dir = resolve_mods_dir(app, state)?;
    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir).map_err(|err| err.to_string())?;
        changed = true;
    }

    Ok(changed)
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
    match serde_json::from_str::<AppState>(&contents) {
        Ok(state) => Ok(state),
        Err(parse_err) => {
            let backup = path.with_extension("json.bak");
            fs::copy(&path, &backup).ok();
            Err(format!(
                "state.json is corrupt ({parse_err}). A backup was saved to {}. Delete state.json to reset.",
                backup.display()
            ))
        }
    }
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

fn extract_archive(app: &AppHandle, archive: &Path, output_dir: &Path) -> Result<(), String> {
    let seven_zip = seven_zip::resolve_7z_executable(app)?;
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
    list_files_in_tree(native_dir)
}

fn list_files_in_tree(root: &Path) -> Result<Vec<String>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let rel = entry
                .path()
                .strip_prefix(root)
                .map_err(|err| err.to_string())?;
            let rel_str = normalize_path(rel);
            if is_safe_rel_path(&rel_str) {
                files.push(rel_str);
            }
        }
    }
    Ok(files)
}

fn is_safe_rel_path(rel: &str) -> bool {
    !rel.is_empty()
        && !rel.contains(':')
        && !rel.starts_with('/')
        && !rel.starts_with('\\')
        && !rel
            .split(['/', '\\'])
            .any(|segment| segment == "..")
}

fn reconcile_mods(app: &AppHandle, state: &mut AppState) -> Result<bool, String> {
    let mods_dir = resolve_mods_dir(app, state)?;
    let mut changed = false;
    let before = state.mods.len();
    state.mods.retain(|entry| {
        mods_dir
            .join(&entry.id)
            .join("nativePC_MT")
            .is_dir()
    });
    if state.mods.len() != before {
        changed = true;
    }

    let known: HashSet<String> = state.mods.iter().map(|entry| entry.id.clone()).collect();
    if !mods_dir.exists() {
        return Ok(changed);
    }

    for entry in fs::read_dir(&mods_dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        if !entry.file_type().map_err(|err| err.to_string())?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if known.contains(&id) {
            continue;
        }
        if let Ok(mod_entry) = rebuild_mod_from_dir(&mods_dir, &id, state.game_path.as_deref()) {
            state.mods.push(mod_entry);
            changed = true;
        }
    }

    Ok(changed)
}

fn rebuild_mod_from_dir(
    mods_dir: &Path,
    id: &str,
    game_path: Option<&str>,
) -> Result<ModEntry, String> {
    let final_mod_dir = mods_dir.join(id);
    let native_dir = final_mod_dir.join("nativePC_MT");
    if !native_dir.is_dir() {
        return Err("nativePC_MT folder is missing.".to_string());
    }

    let modinfo_path = final_mod_dir.join("modinfo.ini");
    let fallback = PathBuf::from(id);
    let (name, author, version, description) = parse_modinfo(&modinfo_path, &fallback);

    let screenshot_path = final_mod_dir.join("screenshot.png");
    let screenshot = if screenshot_path.exists() {
        Some(screenshot_path.to_string_lossy().to_string())
    } else {
        None
    };

    let files = list_mod_files(&native_dir)?;
    let replaces = compute_replacements(game_path, &files)?;

    Ok(ModEntry {
        id: id.to_string(),
        name,
        version,
        author,
        description,
        enabled: false,
        files,
        replaces,
        screenshot,
        source: None,
    })
}

fn rollback_writes(
    native_pc: &Path,
    backup_dir: &Path,
    written_rels: &[String],
) -> Result<(), String> {
    for rel in written_rels.iter().rev() {
        let game_file = join_rel(native_pc, rel)?;
        if let Some(backup) = find_backup_file(backup_dir, rel) {
            restore_file_from_backup(&backup, &game_file)?;
        } else if game_file.is_file() {
            fs::remove_file(&game_file).ok();
        }
    }
    Ok(())
}

fn primary_backup_native(backup_dir: &Path) -> PathBuf {
    backup_dir.join("originals").join("nativePC_MT")
}

fn backup_roots(backup_dir: &Path) -> Vec<PathBuf> {
    vec![
        primary_backup_native(backup_dir),
        backup_dir.join("nativePC_MT"),
        backup_dir.join("originals"),
    ]
}

fn inactive_file_expects_vanilla(state: &AppState, rel: &str) -> bool {
    for entry in &state.mods {
        if entry.enabled {
            continue;
        }
        if entry.files.iter().any(|file| file == rel) && entry.replaces.iter().any(|file| file == rel) {
            return true;
        }
    }
    for removed in &state.pending_removals {
        if removed.files.iter().any(|file| file == rel)
            && removed.replaces.iter().any(|file| file == rel)
        {
            return true;
        }
    }
    false
}

fn restore_file_from_backup(backup: &Path, game_file: &Path) -> Result<(), String> {
    if let Some(parent) = game_file.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    if game_file.exists() {
        fs::remove_file(game_file).map_err(|err| err.to_string())?;
    }
    match fs::rename(backup, game_file) {
        Ok(()) => Ok(()),
        Err(_) => {
            copy_file(backup, game_file)?;
            fs::remove_file(backup).map_err(|err| err.to_string())?;
            Ok(())
        }
    }
}

fn find_backup_file(backup_dir: &Path, rel: &str) -> Option<PathBuf> {
    for root in backup_roots(backup_dir) {
        let candidate = join_rel(&root, rel).ok()?;
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn delete_backup_for_rel(backup_dir: &Path, rel: &str) -> Result<(), String> {
    for root in backup_roots(backup_dir) {
        let candidate = join_rel(&root, rel)?;
        if candidate.is_file() {
            fs::remove_file(&candidate).map_err(|err| err.to_string())?;
            remove_empty_parents(&candidate, &root);
        }
    }
    Ok(())
}

fn remove_empty_parents(file: &Path, root: &Path) {
    let mut dir = file.parent();
    while let Some(current) = dir {
        if current == root {
            break;
        }
        let is_empty = fs::read_dir(current)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false);
        if !is_empty {
            break;
        }
        if fs::remove_dir(current).is_err() {
            break;
        }
        dir = current.parent();
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn join_rel(base: &Path, rel: &str) -> Result<PathBuf, String> {
    for part in rel.split(['/', '\\']) {
        if part == ".." {
            return Err(format!("Invalid path in mod: \"{rel}\""));
        }
    }
    Ok(rel.split(['/', '\\']).fold(base.to_path_buf(), |acc, part| {
        if part.is_empty() || part == "." {
            acc
        } else {
            acc.join(part)
        }
    }))
}

fn compute_replacements(game_path: Option<&str>, files: &[String]) -> Result<Vec<String>, String> {
    let Some(path) = game_path else {
        return Ok(Vec::new());
    };

    let game_path = PathBuf::from(path);
    if !is_valid_game_path(&game_path) {
        return Ok(Vec::new());
    }

    let native_pc = game_path.join("nativePC_MT");
    let mut replaces = Vec::new();
    for rel in files {
        let candidate = join_rel(&native_pc, rel)?;
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
