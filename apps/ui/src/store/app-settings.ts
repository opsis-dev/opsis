import { create } from "zustand";
import { DEFAULT_SETTINGS, mergeSettings, type SettingsPatch } from "@/app/settings/options";
import { on } from "@/app/tauri";
import { commands, events, type ApiServerStatus, type AppSettings } from "@/generated/bindings";
import { loadOrReport } from "@/lib/toast";

type AppSettingsState = {
  settings: AppSettings | null;
  api: ApiServerStatus | null;
};

export const useAppSettings = create<AppSettingsState>()(() => ({ settings: null, api: null }));

export const liveAutoOpen = () =>
  (useAppSettings.getState().settings ?? DEFAULT_SETTINGS).live.autoOpen;

async function reloadSettings(): Promise<void> {
  const settings = await loadOrReport(() => commands.getSettings(), "Couldn't read the settings");
  if (settings) useAppSettings.setState({ settings });
}

async function reloadApiStatus(): Promise<void> {
  const api = await loadOrReport(() => commands.apiStatus(), "Couldn't read the API status");
  if (api) useAppSettings.setState({ api });
}

export function loadedAppSettings(): Promise<AppSettings> {
  const current = useAppSettings.getState().settings;
  if (current) return Promise.resolve(current);
  return new Promise((resolve) => {
    const off = useAppSettings.subscribe(({ settings }) => {
      if (!settings) return;
      off();
      resolve(settings);
    });
  });
}

let started = false;

export function watchAppSettings(): void {
  if (started) return;
  started = true;
  void reloadSettings();
  void reloadApiStatus();
  on(events.settingsChanged, (settings) => useAppSettings.setState({ settings }));
  on(events.apiStatusChanged, (api) => useAppSettings.setState({ api }));
}

export function patchAppSettings(patch: SettingsPatch): Promise<AppSettings> {
  const current = useAppSettings.getState().settings;
  if (current) useAppSettings.setState({ settings: mergeSettings(current, patch) });
  return commands.patchSettings(patch).catch((error: unknown) => {
    void reloadSettings();
    throw error;
  });
}
