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

export type UiState = {
	gamePath?: string | null;
	gamePathValid: boolean;
	modsPath: string;
	backupPath: string;
	mods: ModInfo[];
};

export type ApplyReport = {
	appliedMods: number;
	filesWritten: number;
	filesRemoved: number;
};

export type ModConflict = {
	file: string;
	mods: string[];
};

export const DEFAULT_GAME_PATH =
	"C:\\Program Files (x86)\\Steam\\steamapps\\common\\Resident Evil 5";
