export type ModInfo = {
	id: string;
	name: string;
	version?: string | null;
	author?: string | null;
	description?: string | null;
	enabled: boolean;
	files: string[];
	replaces: string[];
	screenshot?: string | null;
	source?: string | null;
};

export type ApplyConflict = {
	file: string;
	mods: string[];
};

export type UiState = {
	gamePath?: string | null;
	gamePathValid: boolean;
	modsPath: string;
	backupPath: string;
	mods: ModInfo[];
	applyConflicts: ApplyConflict[];
};

export type ApplyReport = {
	appliedMods: number;
	filesWritten: number;
	filesRestored: number;
	filesRemoved: number;
};

export type ModConflict = {
	file: string;
	mods: string[];
};

export const DEFAULT_GAME_PATH =
	"C:\\Program Files (x86)\\Steam\\steamapps\\common\\Resident Evil 5";

export const MOD_ARCHIVE_EXTENSIONS = ["zip", "7z", "rar"] as const;

export function isModArchivePath(path: string): boolean {
	const lower = path.toLowerCase();
	return MOD_ARCHIVE_EXTENSIONS.some((ext) => lower.endsWith(`.${ext}`));
}
