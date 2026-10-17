import { create } from "zustand";
import type { ApiStatus } from "@/shared/ipc";
import { mergeSettings, type AppSettings, type SettingsPatch } from "@/shared/settings";

type AppSettingsState = {
  settings: AppSettings | null;
  api: ApiStatus | null;
};

export const useAppSettings = create<AppSettingsState>()(() => ({ settings: null, api: null }));

export const liveAutoOpen = () => useAppSettings.getState().settings?.live.autoOpen ?? true;

let started = false;

export function watchAppSettings(): void {
  if (started) return;
  started = true;
  void window.demoflow.getSettings().then((settings) => useAppSettings.setState({ settings }));
  void window.demoflow.apiStatus().then((api) => useAppSettings.setState({ api }));
  window.demoflow.onSettingsChanged((settings) => useAppSettings.setState({ settings }));
  window.demoflow.onApiStatus((api) => useAppSettings.setState({ api }));
}

export function patchAppSettings(patch: SettingsPatch): Promise<AppSettings> {
  const current = useAppSettings.getState().settings;
  if (current) useAppSettings.setState({ settings: mergeSettings(current, patch) });
  return window.demoflow.patchSettings(patch).catch((error: unknown) => {
    void window.demoflow.getSettings().then((settings) => useAppSettings.setState({ settings }));
    throw error;
  });
}
