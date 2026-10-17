import { create } from "zustand";
import { reportError } from "@/renderer/lib/toast";
import type { ApiStatus } from "@/shared/ipc";
import {
  DEFAULT_SETTINGS,
  mergeSettings,
  type AppSettings,
  type SettingsPatch,
} from "@/shared/settings";

type AppSettingsState = {
  settings: AppSettings | null;
  api: ApiStatus | null;
};

export const useAppSettings = create<AppSettingsState>()(() => ({ settings: null, api: null }));

export const liveAutoOpen = () =>
  (useAppSettings.getState().settings ?? DEFAULT_SETTINGS).live.autoOpen;

const reloadSettings = () =>
  window.demoflow
    .getSettings()
    .then((settings) => useAppSettings.setState({ settings }))
    .catch((e: unknown) => reportError("Could not read the settings", e));

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
  void window.demoflow.apiStatus().then(
    (api) => useAppSettings.setState({ api }),
    (e: unknown) => reportError("Could not read the API status", e),
  );
  window.demoflow.onSettingsChanged((settings) => useAppSettings.setState({ settings }));
  window.demoflow.onApiStatus((api) => useAppSettings.setState({ api }));
}

export function patchAppSettings(patch: SettingsPatch): Promise<AppSettings> {
  const current = useAppSettings.getState().settings;
  if (current) useAppSettings.setState({ settings: mergeSettings(current, patch) });
  return window.demoflow.patchSettings(patch).catch((error: unknown) => {
    void reloadSettings();
    throw error;
  });
}
