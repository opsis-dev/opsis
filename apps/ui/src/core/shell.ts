import type { DockPanelName, SettingsPage } from "@/generated/bindings";

export const DOCK_PANELS: DockPanelName[] = ["preview", "timeline", "mixer", "run"];

export const DOCK_TITLES: Record<DockPanelName, string> = {
  preview: "Preview",
  timeline: "Timeline",
  mixer: "Mixer",
  run: "Run",
};

export const SETTINGS_PAGES: SettingsPage[] = [
  "general",
  "projects",
  "engine",
  "recording",
  "fonts",
  "export",
  "api",
  "licences",
  "about",
];
