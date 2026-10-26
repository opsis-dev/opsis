import { create } from "zustand";
import { on } from "@/app/tauri";
import { commands, events, type WindowState } from "@/generated/bindings";
import { isMac } from "@/lib/platform";

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
  void commands.getWindowState().then(apply);
  on(events.windowStateChanged, apply);
}
