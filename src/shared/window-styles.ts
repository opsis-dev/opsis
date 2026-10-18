export const WINDOW_STYLES = ["macos", "windows", "linux"] as const;

export type WindowStyle = (typeof WINDOW_STYLES)[number];

export const FRAME_THEMES = ["light", "dark"] as const;

export type FrameTheme = (typeof FRAME_THEMES)[number];

export const WINDOW_STYLE_LABELS: Record<WindowStyle, string> = {
  macos: "macOS",
  windows: "Windows",
  linux: "Linux",
};

const HOST_PLATFORMS: Record<WindowStyle, NodeJS.Platform> = {
  macos: "darwin",
  windows: "win32",
  linux: "linux",
};

export const hostWindowStyle = (platform: NodeJS.Platform): WindowStyle =>
  WINDOW_STYLES.find((style) => HOST_PLATFORMS[style] === platform) ?? "macos";
