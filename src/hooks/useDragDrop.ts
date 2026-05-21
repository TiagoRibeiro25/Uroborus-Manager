import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

type DragDropEvent =
	| { type: "enter"; paths: string[]; position: { x: number; y: number } }
	| { type: "over"; position: { x: number; y: number } }
	| { type: "drop"; paths: string[]; position: { x: number; y: number } }
	| { type: "leave" };

export function useDragDrop(onDrop: (paths: string[]) => void): boolean {
	const [dragActive, setDragActive] = useState<boolean>(false);
	const onDropRef = useRef(onDrop);
	const droppingRef = useRef(false);

	useEffect(() => {
		onDropRef.current = onDrop;
	}, [onDrop]);

	useEffect(() => {
		let unlisten: (() => void) | null = null;
		const appWindow = getCurrentWindow();

		appWindow
			.onDragDropEvent((event: { payload: DragDropEvent }) => {
				switch (event.payload.type) {
					case "enter":
						droppingRef.current = false;
						setDragActive(true);
						break;
					case "drop":
						if (droppingRef.current) break;
						droppingRef.current = true;
						setDragActive(false);
						if (event.payload.paths.length > 0) {
							onDropRef.current(event.payload.paths);
						}
						break;
					case "leave":
						droppingRef.current = false;
						setDragActive(false);
						break;
					default:
						break;
				}
			})
			.then((dispose: () => void) => {
				unlisten = dispose;
			})
			.catch(() => {
				setDragActive(false);
			});

		return () => {
			unlisten?.();
		};
	}, []);

	return dragActive;
}
