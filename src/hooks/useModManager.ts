import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
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
			setBusy(true);
			setError(null);
			try {
				for (const archivePath of paths) {
					await invoke("import_mod_archive", { archivePath });
				}
				await refreshState();
			} catch (err) {
				setError(String(err));
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
		} catch (err) {
			setError(String(err));
		} finally {
			setBusy(false);
		}
	}, []);

	const gamePathLabel = state?.gamePath ?? DEFAULT_GAME_PATH;
	const modsPath = state?.modsPath ?? "";
	const backupPath = state?.backupPath ?? "";

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
	};
}
