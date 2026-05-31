type TopHeaderProps = {
	gamePathValid: boolean;
	busy: boolean;
	onPickGamePath: () => void;
	onLaunchGame: () => void;
};

export default function TopHeader({
	gamePathValid,
	busy,
	onPickGamePath,
	onLaunchGame,
}: TopHeaderProps) {
	return (
		<header className="mb-6 flex flex-col gap-4 rounded-2xl border border-amber-400/20 bg-zinc-900/70 p-5 shadow-[0_16px_40px_rgba(0,0,0,0.45)] backdrop-blur-xl lg:flex-row lg:items-center lg:justify-between">
			<div>
				<p className="m-0 font-['Oxanium'] text-lg tracking-widest">
					Uroborus Manager
				</p>
				<p className="m-0 text-sm text-amber-100/70">
					Resident Evil 5 mod control hub
				</p>
			</div>
			<div className="flex flex-wrap items-center gap-3">
				<span
					className={`rounded-full border px-4 py-1 text-[11px] uppercase tracking-[0.2em] ${
						gamePathValid
							? "border-amber-300/50 bg-amber-300/15 text-amber-100"
							: "border-amber-500/50 bg-amber-500/10 text-amber-200"
					}`}
				>
					{gamePathValid ? "Game path verified" : "Game path missing"}
				</span>
				<button
					className="rounded-full border border-amber-300/30 px-4 py-2 text-[11px] uppercase tracking-[0.2em] text-amber-100/80 transition hover:border-amber-200/60 hover:text-amber-100"
					onClick={onPickGamePath}
					disabled={busy}
				>
					Select Game Folder
				</button>
				<button
					className="rounded-full bg-amber-300 px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.2em] text-zinc-900 transition hover:bg-amber-200 disabled:opacity-40"
					onClick={onLaunchGame}
					disabled={busy || !gamePathValid}
				>
					Launch Game
				</button>
			</div>
		</header>
	);
}
