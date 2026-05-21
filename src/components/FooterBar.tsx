import { ApplyReport } from "../types/mods";

type FooterBarProps = {
	gamePath: string;
	modsPath: string;
	backupPath: string;
	busy: boolean;
	pendingChanges: boolean;
	gamePathValid: boolean;
	applyReport: ApplyReport | null;
	onPickModsPath: () => void;
	onPickBackupPath: () => void;
	onApply: () => void;
};

export default function FooterBar({
	gamePath,
	modsPath,
	backupPath,
	busy,
	pendingChanges,
	gamePathValid,
	applyReport,
	onPickModsPath,
	onPickBackupPath,
	onApply,
}: FooterBarProps) {
	return (
		<footer className="mt-6 grid gap-4 rounded-2xl border border-amber-400/20 bg-zinc-900/70 p-5 shadow-[0_16px_40px_rgba(0,0,0,0.4)] backdrop-blur sm:grid-cols-2 xl:grid-cols-[1.1fr_1.1fr_1.1fr_1fr]">
			<div>
				<p className="m-0 text-xs uppercase tracking-[0.2em] text-amber-100/60">
					Game path
				</p>
				<span className="mt-2 block break-all text-sm text-amber-100/80">
					{gamePath}
				</span>
			</div>
			<div>
				<p className="m-0 text-xs uppercase tracking-[0.2em] text-amber-100/60">
					Mods path
				</p>
				<span className="mt-2 block break-all text-sm text-amber-100/80">
					{modsPath}
				</span>
				<button
					className="mt-3 rounded-full border border-amber-300/30 px-4 py-2 text-[11px] uppercase tracking-[0.2em] text-amber-100/80 transition hover:border-amber-200/60 hover:text-amber-100"
					onClick={onPickModsPath}
					disabled={busy}
				>
					Change Mods Path
				</button>
			</div>
			<div>
				<p className="m-0 text-xs uppercase tracking-[0.2em] text-amber-100/60">
					Backups path
				</p>
				<span className="mt-2 block break-all text-sm text-amber-100/80">
					{backupPath}
				</span>
				<button
					className="mt-3 rounded-full border border-amber-300/30 px-4 py-2 text-[11px] uppercase tracking-[0.2em] text-amber-100/80 transition hover:border-amber-200/60 hover:text-amber-100"
					onClick={onPickBackupPath}
					disabled={busy}
				>
					Change Backups Path
				</button>
			</div>
			<div className="flex flex-col gap-3 sm:col-span-2 xl:col-span-1">
				{applyReport ? (
					<p className="m-0 text-xs text-amber-100/70">
						Applied {applyReport.appliedMods} mods, wrote{" "}
						{applyReport.filesWritten} files, removed{" "}
						{applyReport.filesRemoved} files.
					</p>
				) : (
					<p className="m-0 text-xs text-amber-100/70">Awaiting changes.</p>
				)}
				<button
					className={`rounded-full bg-amber-300 px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.2em] text-zinc-900 transition hover:bg-amber-200 ${
						pendingChanges ? "animate-pulse" : ""
					}`}
					onClick={onApply}
					disabled={busy || !gamePathValid}
				>
					Apply Enabled Mods
				</button>
			</div>
		</footer>
	);
}
