export const WINDOW_STYLES = ["macos", "windows", "linux"] as const;

export type WindowStyle = (typeof WINDOW_STYLES)[number];

export const FRAME_THEMES = ["light", "dark"] as const;

export type FrameTheme = (typeof FRAME_THEMES)[number];

export const DEFAULT_WINDOW_STYLE: WindowStyle = "macos";

export const DEFAULT_FRAME_THEME: FrameTheme = "light";

export const DEFAULT_ADDRESS_BAR = true;

export const WINDOW_STYLE_LABELS: Record<WindowStyle, string> = {
  macos: "macOS",
  windows: "Windows",
  linux: "Linux",
};

const HOST_STYLES: Partial<Record<NodeJS.Platform, WindowStyle>> = {
  darwin: "macos",
  win32: "windows",
  linux: "linux",
};

export const hostWindowStyle = (platform: NodeJS.Platform): WindowStyle =>
  HOST_STYLES[platform] ?? DEFAULT_WINDOW_STYLE;
