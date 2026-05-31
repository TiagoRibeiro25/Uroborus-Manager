import Navbar from "./components/Navbar";
import FooterBar from "./components/FooterBar";
import ModDetails from "./components/ModDetails";
import ModList from "./components/ModList";
import OverlayLayers from "./components/OverlayLayers";
import TopHeader from "./components/TopHeader";
import { useDragDrop } from "./hooks/useDragDrop";
import { useModManager } from "./hooks/useModManager";

export default function App() {
	const {
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
		disableAllMods,
		reorderMod,
		openGameFolder,
		openModsLibrary,
		openModFolder,
		launchGame,
		applyConflicts,
		hasApplyConflicts,
	} = useModManager();

	const dragActive = useDragDrop(importMods);
	const gamePathValid = Boolean(state?.gamePathValid);
	const showGamePathPrompt = Boolean(state && !state.gamePathValid);
	const selectedModIndex = selectedId
		? mods.findIndex((mod) => mod.id === selectedId)
		: -1;

	return (
		<div className="relative min-h-screen overflow-hidden bg-zinc-950 font-['Spline_Sans'] text-amber-50">
			<Navbar />
			<div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_15%_20%,rgba(251,191,36,0.16),transparent_55%)]" />
			<div className="pointer-events-none absolute -top-32 -left-20 h-105 w-105 rounded-full bg-amber-300/10 blur-[120px]" />
			<div className="pointer-events-none absolute -bottom-40 -right-30 h-105 w-105 rounded-full bg-amber-200/10 blur-[140px]" />
			<div className="pointer-events-none absolute inset-0 bg-[linear-gradient(rgba(251,191,36,0.06)_1px,transparent_1px),linear-gradient(90deg,rgba(251,191,36,0.06)_1px,transparent_1px)] bg-size-[32px_32px] opacity-70" />

			<div className="relative z-10 px-6 pb-10 pt-20">
				<TopHeader
					gamePathValid={gamePathValid}
					busy={busy}
					onPickGamePath={pickGamePath}
					onLaunchGame={launchGame}
				/>
				<main className="grid gap-6 lg:grid-cols-[320px_1fr]">
					<ModList
						mods={mods}
						selectedId={selectedId}
						busy={busy}
						onSelect={(id) => setSelectedId(id)}
						onToggle={toggleMod}
						onRemove={removeMod}
						onAdd={pickMods}
						onDisableAll={disableAllMods}
						onReorder={reorderMod}
					/>
					<ModDetails
						mod={selectedMod}
						conflicts={selectedConflicts}
						modIndex={selectedModIndex >= 0 ? selectedModIndex : 0}
						modCount={mods.length}
						onOpenModFolder={openModFolder}
					/>
				</main>
				<FooterBar
					gamePath={gamePathLabel}
					modsPath={modsPath}
					backupPath={backupPath}
					busy={busy}
					pendingChanges={pendingChanges}
					gamePathValid={gamePathValid}
					hasApplyConflicts={hasApplyConflicts}
					applyConflicts={applyConflicts}
					applyReport={applyReport}
					onPickModsPath={pickModsPath}
					onPickBackupPath={pickBackupPath}
					onOpenGameFolder={openGameFolder}
					onOpenModsLibrary={openModsLibrary}
					onApply={applyMods}
				/>
			</div>

			<OverlayLayers
				dragActive={dragActive}
				showGamePathPrompt={showGamePathPrompt}
				onPickGamePath={pickGamePath}
				error={error}
			/>
		</div>
	);
}
