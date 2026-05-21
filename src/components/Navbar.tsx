import { getCurrentWindow } from "@tauri-apps/api/window";
import { IoMdClose } from "react-icons/io";
import { VscChromeMaximize, VscChromeMinimize } from "react-icons/vsc";

export default function Navbar() {
	const appWindow = getCurrentWindow();

	const handleMaximize = async () => {
		const isMaximized = await appWindow.isMaximized();
		if (isMaximized) {
			await appWindow.unmaximize();
		} else {
			await appWindow.maximize();
		}
	};

	return (
		<nav
			className="fixed left-0 right-0 top-0 z-40 flex items-center justify-between border-b border-amber-400/20 bg-zinc-950/60 px-4 py-2 text-amber-50 backdrop-blur"
			data-tauri-drag-region
		>
			<div className="flex items-center gap-3" data-tauri-drag-region>
				<p className="mt-1 font-['Oxanium'] text-xs uppercase tracking-[0.35em] text-amber-200">
					Uroborus Manager
				</p>
				<span className="text-[10px] uppercase tracking-[0.3em] text-amber-100/60">
					Made by GOLD
				</span>
			</div>
			<div className="flex items-center gap-2" data-tauri-drag-region>
				<button
					className="flex h-7 w-7 items-center justify-center rounded-md border border-amber-400/10 text-amber-100 transition hover:border-amber-300/40 hover:text-amber-50"
					onClick={() => appWindow.minimize()}
					data-tauri-drag-region="false"
					title="Minimize"
				>
					<VscChromeMinimize className="h-4 w-4" />
				</button>
				<button
					className="flex h-7 w-7 items-center justify-center rounded-md border border-amber-400/10 text-amber-100 transition hover:border-amber-300/40 hover:text-amber-50"
					onClick={handleMaximize}
					data-tauri-drag-region="false"
					title="Maximize"
				>
					<VscChromeMaximize className="h-4 w-4" />
				</button>
				<button
					className="flex h-7 w-7 items-center justify-center rounded-md border border-amber-400/10 text-amber-100 transition hover:border-amber-300/60 hover:bg-amber-500/20 hover:text-amber-50"
					onClick={() => appWindow.close()}
					data-tauri-drag-region="false"
					title="Close"
				>
					<IoMdClose className="h-4 w-4" />
				</button>
			</div>
		</nav>
	);
}
