import { create } from "zustand";
import type { WindowState } from "@/shared/ipc";
import { isMac } from "@/renderer/lib/platform";

export const useWindowState = create<WindowState>()(() => ({
  maximized: false,
  fullscreen: false,
  focused: true,
}));

let started = false;

export function watchWindowState(): void {
  if (started || isMac) return;
  started = true;
  const apply = (state: WindowState) => useWindowState.setState(state);
  void window.demoflow.getWindowState().then(apply);
  window.demoflow.onWindowState(apply);
}
