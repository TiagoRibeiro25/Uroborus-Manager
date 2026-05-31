import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import {
	ApplyConflict,
	ApplyReport,
	DEFAULT_GAME_PATH,
	ModConflict,
	ModInfo,
	UiState,
} from "../types/mods";

export type ModManagerState = {
	state: UiState | null;
	mods: ModInfo[];
	selectedId: string | null;
	selectedMod: ModInfo | null;
	selectedConflicts: ModConflict[];
	pendingChanges: boolean;
	busy: boolean;
	error: string | null;
	applyReport: ApplyReport | null;
	applyConflicts: ApplyConflict[];
	hasApplyConflicts: boolean;
	gamePathLabel: string;
	modsPath: string;
	backupPath: string;
	setSelectedId: (id: string | null) => void;
	pickMods: () => Promise<void>;
	pickGamePath: () => Promise<void>;
	pickModsPath: () => Promise<void>;
	pickBackupPath: () => Promise<void>;
	toggleMod: (mod: ModInfo) => Promise<void>;
	removeMod: (mod: ModInfo) => Promise<void>;
	applyMods: () => Promise<void>;
	importMods: (paths: string[]) => Promise<void>;
	disableAllMods: () => Promise<void>;
	reorderMod: (mod: ModInfo, direction: "up" | "down") => Promise<void>;
	openGameFolder: () => Promise<void>;
	openModsLibrary: () => Promise<void>;
	openModFolder: (mod: ModInfo) => Promise<void>;
	launchGame: () => Promise<void>;
};

export function useModManager(): ModManagerState {
	const [state, setState] = useState<UiState | null>(null);
	const [selectedId, setSelectedId] = useState<string | null>(null);
	const [pendingChanges, setPendingChanges] = useState(false);
	const [busy, setBusy] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [applyReport, setApplyReport] = useState<ApplyReport | null>(null);

	const refreshState = useCallback(async () => {
		try {
			const nextState = await invoke<UiState>("get_state");
			setState(nextState);
			setError(null);
		} catch (err) {
			setError(String(err));
		}
	}, []);

	useEffect(() => {
		refreshState();
	}, [refreshState]);

	useEffect(() => {
		if (!state) return;
		if (!selectedId && state.mods.length > 0) {
			setSelectedId(state.mods[0].id);
		}
	}, [state, selectedId]);

	const mods = state?.mods ?? [];
	const applyConflicts = state?.applyConflicts ?? [];
	const hasApplyConflicts = applyConflicts.length > 0;

	const selectedMod = useMemo(() => {
		if (!state || !selectedId) return null;
		return state.mods.find((mod) => mod.id === selectedId) ?? null;
	}, [state, selectedId]);

	const conflictMap = useMemo(() => {
		const map = new Map<string, string[]>();
		mods
			.filter((mod) => mod.enabled)
			.forEach((mod) => {
				mod.files.forEach((file) => {
					const list = map.get(file) ?? [];
					list.push(mod.id);
					map.set(file, list);
				});
			});
		return map;
	}, [mods]);

	const selectedConflicts = useMemo<ModConflict[]>(() => {
		if (!selectedMod) return [];
		const byId = new Map(mods.map((mod) => [mod.id, mod.name]));
		return selectedMod.files
			.filter((file) => (conflictMap.get(file) ?? []).length > 1)
			.slice(0, 6)
			.map((file) => ({
				file,
				mods: (conflictMap.get(file) ?? []).map(
					(modId) => byId.get(modId) ?? modId,
					),
			}));
	}, [selectedMod, conflictMap, mods]);

	const importMods = useCallback(
		async (paths: string[]) => {
			if (!paths.length) return;
			setBusy(true);
			setError(null);
			const imported: string[] = [];
			try {
				for (const archivePath of paths) {
					await invoke("import_mod_archive", { archivePath });
					imported.push(archivePath);
				}
				await refreshState();
			} catch (err) {
				const suffix =
					imported.length > 0
						? ` ${imported.length} archive(s) were imported before the failure.`
						: "";
				setError(`${String(err)}${suffix}`);
				if (imported.length > 0) {
					await refreshState();
				}
			} finally {
				setBusy(false);
			}
		},
		[refreshState],
	);

	const pickMods = useCallback(async () => {
		const selection = await open({
			multiple: true,
			directory: false,
			filters: [
				{
					name: "Mod archives",
					extensions: ["zip", "7z", "rar"],
				},
			],
		});
		if (!selection) return;
		const paths = Array.isArray(selection) ? selection : [selection];
		await importMods(paths);
	}, [importMods]);

	const pickGamePath = useCallback(async () => {
		const selection = await open({ directory: true, multiple: false });
		if (!selection || Array.isArray(selection)) return;
		setBusy(true);
		setError(null);
		try {
			const nextState = await invoke<UiState>("set_game_path", {
				path: selection,
			});
			setState(nextState);
		} catch (err) {
			setError(String(err));
		} finally {
			setBusy(false);
		}
	}, []);

	const pickModsPath = useCallback(async () => {
		const selection = await open({ directory: true, multiple: false });
		if (!selection || Array.isArray(selection)) return;
		setBusy(true);
		setError(null);
		try {
			const nextState = await invoke<UiState>("set_mods_path", {
				path: selection,
			});
			setState(nextState);
		} catch (err) {
			setError(String(err));
		} finally {
			setBusy(false);
		}
	}, []);

	const pickBackupPath = useCallback(async () => {
		const selection = await open({ directory: true, multiple: false });
		if (!selection || Array.isArray(selection)) return;
		setBusy(true);
		setError(null);
		try {
			const nextState = await invoke<UiState>("set_backup_path", {
				path: selection,
			});
			setState(nextState);
		} catch (err) {
			setError(String(err));
		} finally {
			setBusy(false);
		}
	}, []);

	const toggleMod = useCallback(async (mod: ModInfo) => {
		setBusy(true);
		setError(null);
		try {
			const nextState = await invoke<UiState>("set_mod_enabled", {
				id: mod.id,
				enabled: !mod.enabled,
			});
			setState(nextState);
			setPendingChanges(true);
			setApplyReport(null);
		} catch (err) {
			setError(String(err));
		} finally {
			setBusy(false);
		}
	}, []);

	const removeMod = useCallback(
		async (mod: ModInfo) => {
			const confirmed = window.confirm(
				`Remove ${mod.name}? The extracted files will be deleted.`,
			);
			if (!confirmed) return;
			setBusy(true);
			setError(null);
			try {
				const nextState = await invoke<UiState>("remove_mod", { id: mod.id });
				setState(nextState);
				setPendingChanges(true);
				setApplyReport(null);
				if (selectedId === mod.id) {
					setSelectedId(nextState.mods[0]?.id ?? null);
				}
			} catch (err) {
				setError(String(err));
			} finally {
				setBusy(false);
			}
		},
		[selectedId],
	);

	const applyMods = useCallback(async () => {
		setBusy(true);
		setError(null);
		try {
			const report = await invoke<ApplyReport>("apply_mods");
			setApplyReport(report);
			setPendingChanges(false);
			await refreshState();
		} catch (err) {
			setError(String(err));
		} finally {
			setBusy(false);
		}
	}, [refreshState]);

	const disableAllMods = useCallback(async () => {
		const enabledCount = mods.filter((mod) => mod.enabled).length;
		if (!enabledCount) return;
		setBusy(true);
		setError(null);
		try {
			const nextState = await invoke<UiState>("disable_all_mods");
			setState(nextState);
			setPendingChanges(true);
			setApplyReport(null);
		} catch (err) {
			setError(String(err));
		} finally {
			setBusy(false);
		}
	}, [mods]);

	const reorderMod = useCallback(async (mod: ModInfo, direction: "up" | "down") => {
		setBusy(true);
		setError(null);
		try {
			const nextState = await invoke<UiState>("reorder_mod", {
				id: mod.id,
				direction,
			});
			setState(nextState);
			setPendingChanges(true);
			setApplyReport(null);
		} catch (err) {
			setError(String(err));
		} finally {
			setBusy(false);
		}
	}, []);

	const openGameFolder = useCallback(async () => {
		const path = state?.gamePath;
		if (!path) {
			setError("Set a valid game folder first.");
			return;
		}
		try {
			await openPath(path);
		} catch (err) {
			setError(String(err));
		}
	}, [state?.gamePath]);

	const gamePathLabel = state?.gamePath ?? DEFAULT_GAME_PATH;
	const modsPath = state?.modsPath ?? "";
	const backupPath = state?.backupPath ?? "";

	const openModsLibrary = useCallback(async () => {
		if (!modsPath) return;
		try {
			await openPath(modsPath);
		} catch (err) {
			setError(String(err));
		}
	}, [modsPath]);

	const openModFolder = useCallback(
		async (mod: ModInfo) => {
			if (!modsPath) return;
			try {
				await openPath(`${modsPath}/${mod.id}`);
			} catch (err) {
				setError(String(err));
			}
		},
		[modsPath],
	);

	const launchGame = useCallback(async () => {
		setBusy(true);
		setError(null);
		try {
			await invoke("launch_game");
		} catch (err) {
			setError(String(err));
		} finally {
			setBusy(false);
		}
	}, []);

	return {
		state,
		mods,
		selectedId,
		selectedMod,
		selectedConflicts,
		pendingChanges,
		busy,
		error,
		applyReport,
		applyConflicts,
		hasApplyConflicts,
		gamePathLabel,
		modsPath,
		backupPath,
		setSelectedId,
		pickMods,
		pickGamePath,
		pickModsPath,
		pickBackupPath,
		toggleMod,
		removeMod,
		applyMods,
		importMods,
		disableAllMods,
		reorderMod,
		openGameFolder,
		openModsLibrary,
		openModFolder,
		launchGame,
	};
}
