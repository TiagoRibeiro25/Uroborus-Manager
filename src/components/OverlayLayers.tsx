type OverlayLayersProps = {
	dragActive: boolean;
	showGamePathPrompt: boolean;
	onPickGamePath: () => void;
	error: string | null;
};

export default function OverlayLayers({
	dragActive,
	showGamePathPrompt,
	onPickGamePath,
	error,
}: OverlayLayersProps) {
	return (
		<>
			{dragActive && (
				<div className="fixed inset-0 z-40 grid place-items-center bg-zinc-950/80 text-center backdrop-blur">
					<div className="rounded-2xl border border-dashed border-amber-400/50 bg-zinc-900/90 px-10 py-8">
						<p className="m-0 font-['Oxanium'] text-lg tracking-[0.2em]">
							Drop archives to import
						</p>
						<span className="mt-2 block text-xs uppercase tracking-[0.3em] text-amber-100/70">
							ZIP, 7Z, or RAR
						</span>
					</div>
				</div>
			)}

			{showGamePathPrompt && (
				<div className="fixed inset-0 z-50 grid place-items-center bg-zinc-950/90 px-6">
					<div className="max-w-md rounded-2xl border border-amber-400/30 bg-zinc-900/95 p-8 text-center shadow-[0_20px_50px_rgba(0,0,0,0.6)]">
						<h2 className="m-0 font-['Oxanium'] text-xl tracking-[0.2em]">
							Locate Resident Evil 5
						</h2>
						<p className="mt-4 text-sm text-amber-100/70">
							We could not verify the default path. Select the folder that
							 contains re5dx9.exe and nativePC_MT.
						</p>
						<button
							className="mt-6 rounded-full bg-amber-300 px-5 py-2 text-[11px] font-semibold uppercase tracking-[0.2em] text-zinc-900 transition hover:bg-amber-200"
							onClick={onPickGamePath}
						>
							Select Game Folder
						</button>
					</div>
				</div>
			)}

			{error && (
				<div className="fixed bottom-6 right-6 z-50 rounded-xl border border-amber-500/40 bg-zinc-900/90 px-4 py-3 text-sm text-amber-100 shadow-[0_18px_40px_rgba(0,0,0,0.4)]">
					{error}
				</div>
			)}
		</>
	);
}
