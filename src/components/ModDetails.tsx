import { useMemo } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ModConflict, ModInfo } from "../types/mods";

type ModDetailsProps = {
	mod: ModInfo | null;
	conflicts: ModConflict[];
	modIndex: number;
	modCount: number;
	onOpenModFolder: (mod: ModInfo) => void;
};

export default function ModDetails({
	mod,
	conflicts,
	modIndex,
	modCount,
	onOpenModFolder,
}: ModDetailsProps) {
	const screenshotSrc = useMemo(() => {
		if (!mod?.screenshot) return null;
		try {
			return convertFileSrc(mod.screenshot);
		} catch {
			return null;
		}
	}, [mod?.screenshot]);

	return (
		<section className="rounded-2xl border border-amber-400/20 bg-zinc-900/70 p-5 shadow-[0_16px_40px_rgba(0,0,0,0.4)] backdrop-blur">
			{mod ? (
				<div className="space-y-6">
					<div className="flex flex-col gap-4 lg:flex-row lg:items-start">
						{screenshotSrc ? (
							<img
								src={screenshotSrc}
								alt={`${mod.name} preview`}
								className="h-40 w-full max-w-xs shrink-0 rounded-xl border border-amber-400/20 object-cover lg:h-48"
							/>
						) : (
							<div className="flex h-40 w-full max-w-xs shrink-0 items-center justify-center rounded-xl border border-dashed border-amber-400/30 bg-zinc-950/60 text-xs uppercase tracking-[0.2em] text-amber-100/50 lg:h-48">
								No screenshot
							</div>
						)}
						<div className="min-w-0 flex-1">
							<h3 className="m-0 font-['Oxanium'] text-xl tracking-widest">
								{mod.name}
							</h3>
							<div className="mt-3 flex flex-wrap gap-2 text-[11px] uppercase tracking-[0.2em]">
								<span className="rounded-full border border-amber-400/30 bg-amber-300/10 px-3 py-1 text-amber-100">
									{mod.version ?? "No version"}
								</span>
								<span className="rounded-full border border-amber-400/30 bg-amber-300/10 px-3 py-1 text-amber-100">
									{mod.author ?? "Unknown author"}
								</span>
								<span className="rounded-full border border-amber-400/30 bg-amber-300/10 px-3 py-1 text-amber-100">
									{mod.files.length} files
								</span>
								<span className="rounded-full border border-amber-400/30 bg-amber-300/10 px-3 py-1 text-amber-100">
									Priority {modIndex + 1}/{modCount}
								</span>
							</div>
							<p className="mt-4 text-sm text-amber-100/70">
								{mod.description ?? "No description was provided for this mod."}
							</p>
							<div className="mt-4 flex flex-wrap gap-2">
								<button
									type="button"
									className="rounded-full border border-amber-300/30 px-4 py-2 text-[11px] uppercase tracking-[0.2em] text-amber-100/80 transition hover:border-amber-200/60 hover:text-amber-100"
									onClick={() => onOpenModFolder(mod)}
								>
									Open Mod Folder
								</button>
								{mod.source && (
									<p className="m-0 self-center text-xs text-amber-100/50">
										Source: {mod.source.split(/[/\\]/).pop()}
									</p>
								)}
							</div>
						</div>
					</div>

					<div className="grid gap-4 lg:grid-cols-2">
						<div className="rounded-xl border border-amber-400/20 bg-zinc-950/60 p-4">
							<h4 className="m-0 font-['Oxanium'] text-sm tracking-[0.15em]">
								Replacement Scan
							</h4>
							<p className="mt-2 text-sm text-amber-100/70">
								{mod.replaces.length} game files will be replaced when enabled.
							</p>
							{mod.replaces.length > 0 ? (
								<ul className="mt-3 space-y-2 text-xs text-amber-100/70">
									{mod.replaces.slice(0, 5).map((file) => (
										<li key={file}>{file}</li>
									))}
								</ul>
							) : (
								<p className="mt-3 text-xs text-amber-100/60">
									No direct replacements.
								</p>
							)}
						</div>
						<div className="rounded-xl border border-amber-400/20 bg-zinc-950/60 p-4">
							<h4 className="m-0 font-['Oxanium'] text-sm tracking-[0.15em]">
								Conflicts
							</h4>
							<p className="mt-2 text-sm text-amber-100/70">
								{conflicts.length
									? "Overlaps with other enabled mods — Apply is blocked until resolved."
									: "No conflicts detected with enabled mods."}
							</p>
							{conflicts.length > 0 && (
								<ul className="mt-3 space-y-2 text-xs text-amber-100/70">
									{conflicts.map((conflict) => (
										<li key={conflict.file}>
											{conflict.file}
											<span className="mt-1 block text-[10px] uppercase tracking-[0.2em] text-amber-200/70">
												Shared with {conflict.mods.join(", ")}
											</span>
										</li>
									))}
								</ul>
							)}
						</div>
					</div>
				</div>
			) : (
				<div className="rounded-xl border border-dashed border-amber-400/30 p-6 text-center text-amber-100/70">
					Select a mod to view details.
				</div>
			)}
		</section>
	);
}
