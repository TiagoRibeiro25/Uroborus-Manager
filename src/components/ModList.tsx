import { ModInfo } from "../types/mods";

type ModListProps = {
	mods: ModInfo[];
	selectedId: string | null;
	busy: boolean;
	onSelect: (id: string) => void;
	onToggle: (mod: ModInfo) => void;
	onRemove: (mod: ModInfo) => void;
	onAdd: () => void;
};

export default function ModList({
	mods,
	selectedId,
	busy,
	onSelect,
	onToggle,
	onRemove,
	onAdd,
}: ModListProps) {
	return (
		<aside className="rounded-2xl border border-amber-400/20 bg-zinc-900/70 p-5 shadow-[0_16px_40px_rgba(0,0,0,0.4)] backdrop-blur">
			<div className="mb-5 flex items-center justify-between">
				<div>
					<h2 className="m-0 font-['Oxanium'] text-base tracking-[0.12em]">Library</h2>
					<p className="mt-2 text-xs text-amber-100/60">{mods.length} mods indexed</p>
				</div>
				<button
					className="rounded-full bg-amber-300 px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.2em] text-zinc-900 transition hover:bg-amber-200"
					onClick={onAdd}
					disabled={busy}
				>
					Add Mods
				</button>
			</div>

			<div className="space-y-3">
				{mods.length ? (
					mods.map((mod) => (
						<div
							key={mod.id}
							className={`flex w-full items-center justify-between gap-4 rounded-xl border p-4 text-left transition ${
								selectedId === mod.id
									? "border-amber-300/60 bg-zinc-900/80 shadow-[0_0_0_1px_rgba(251,191,36,0.2)]"
									: "border-amber-400/10 bg-zinc-950/60 hover:-translate-y-0.5 hover:border-amber-300/40"
							}`}
							role="button"
							tabIndex={0}
							onClick={() => onSelect(mod.id)}
							onKeyDown={(event) => {
								if (event.key === "Enter" || event.key === " ") {
									event.preventDefault();
									onSelect(mod.id);
								}
							}}
						>
							<div>
								<p className="m-0 font-semibold">{mod.name}</p>
								<p className="mt-1 text-xs text-amber-100/60">
									{mod.author ? `by ${mod.author}` : "Unknown author"}
								</p>
								<p className="mt-1 text-xs text-amber-100/60">{mod.files.length} files</p>
							</div>
							<div className="flex flex-col items-end gap-2">
								<div className="flex items-center gap-2 text-[10px] uppercase tracking-[0.25em] text-amber-100/60">
									<span>{mod.enabled ? "Enabled" : "Disabled"}</span>
									<button
										className={`relative h-6 w-12 rounded-full border transition ${
											mod.enabled
												? "border-amber-300/60 bg-amber-300/30"
												: "border-amber-100/20 bg-zinc-900/60"
										}`}
										onClick={(event) => {
											event.stopPropagation();
											onToggle(mod);
										}}
										disabled={busy}
										aria-pressed={mod.enabled}
										aria-label={mod.enabled ? "Disable mod" : "Enable mod"}
									>
										<span
											className={`absolute top-1/2 h-4 w-4 -translate-y-1/2 rounded-full transition ${
												mod.enabled
													? "translate-x-1 bg-amber-200"
													: "-translate-x-5 bg-amber-100/60"
											}`}
										/>
									</button>
								</div>
								<button
									className="text-[10px] uppercase tracking-[0.25em] text-amber-100/60 transition hover:text-amber-100"
									onClick={(event) => {
										event.stopPropagation();
										onRemove(mod);
									}}
									disabled={busy}
								>
									Remove
								</button>
							</div>
						</div>
					))
				) : (
					<div className="rounded-xl border border-dashed border-amber-400/30 p-6 text-center text-amber-100/70">
						<p className="m-0">No mods yet.</p>
						<p className="mt-2 text-sm">Drag archives here or use Add Mods.</p>
					</div>
				)}
			</div>
		</aside>
	);
}
