import { fontOr } from "#video/common/fonts.ts";
import type { ElementSettings, Theme } from "#video/generated/spec.ts";

const STORY_PALETTES = {
  dark: {
    page: "#080810",
    hookBg: "#0c0c14",
    tickerBg: "#0a1014",
    timelineBg: "#0c0c14",
    window: "#0B0C11",
    border: "rgba(255,255,255,0.09)",
    fg: "#D8DCE4",
    dim: "#7A828F",
    faint: "#4A5160",
    ink: "#fafafa",
    label: "rgba(250,250,250,0.62)",
    axis: "rgba(250,250,250,0.42)",
    grid: "rgba(250,250,250,0.08)",
    track: "rgba(250,250,250,0.05)",
    up: "#2dd4bf",
    down: "#f87171",
  },
  light: {
    page: "#F4F4F2",
    hookBg: "#F4F4F2",
    tickerBg: "#F4F4F2",
    timelineBg: "#F4F4F2",
    window: "#FFFFFF",
    border: "rgba(0,0,0,0.10)",
    fg: "#22252B",
    dim: "#6B7280",
    faint: "#A0A6B0",
    ink: "#22252B",
    label: "rgba(34,37,43,0.66)",
    axis: "rgba(34,37,43,0.48)",
    grid: "rgba(34,37,43,0.10)",
    track: "rgba(34,37,43,0.06)",
    up: "#0f766e",
    down: "#dc2626",
  },
} as const;

type StoryPalette = { [K in keyof (typeof STORY_PALETTES)["dark"]]: string };

function storyPalette(theme: Theme): StoryPalette {
  const base: StoryPalette = STORY_PALETTES[theme.mode];
  const picked = Object.entries(base).map(([key, value]) => [key, theme.colors[key] ?? value]);
  return Object.fromEntries(picked) as StoryPalette;
}

export type StoryLook = {
  palette: StoryPalette;
  font: string;
  textSize: number;
  glow: number;
  eyebrowCaps: boolean;
};

export function storyLook(settings: ElementSettings["data-story"]): StoryLook {
  return {
    palette: storyPalette(settings.theme),
    font: fontOr(settings.fontFamily, "main"),
    textSize: settings.textSize,
    glow: settings.glow,
    eyebrowCaps: settings.eyebrowCaps,
  };
}
