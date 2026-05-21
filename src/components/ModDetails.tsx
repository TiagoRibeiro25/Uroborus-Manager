import { ModConflict, ModInfo } from "../types/mods";

type ModDetailsProps = {
	mod: ModInfo | null;
	conflicts: ModConflict[];
};

export default function ModDetails({ mod, conflicts }: ModDetailsProps) {
	return (
		<section className="rounded-2xl border border-amber-400/20 bg-zinc-900/70 p-5 shadow-[0_16px_40px_rgba(0,0,0,0.4)] backdrop-blur">
			{mod ? (
				<div className="space-y-6">
					<div>
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
						</div>
						<p className="mt-4 text-sm text-amber-100/70">
							{mod.description ?? "No description was provided for this mod."}
						</p>
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
									? "Potential overlaps detected."
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
