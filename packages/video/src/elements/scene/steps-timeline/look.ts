import { fontOr } from "#video/common/fonts.ts";
import type { ElementSettings, Theme } from "#video/generated/spec.ts";

const STEPS_PALETTES = {
  dark: {
    page: "#0c0c14",
    window: "#0B0C11",
    border: "rgba(255,255,255,0.09)",
    fg: "#D8DCE4",
    dim: "#7A828F",
    faint: "#4A5160",
  },
  light: {
    page: "#F4F4F2",
    window: "#FFFFFF",
    border: "rgba(0,0,0,0.10)",
    fg: "#22252B",
    dim: "#6B7280",
    faint: "#A0A6B0",
  },
} as const;

type StepsPalette = { [K in keyof (typeof STEPS_PALETTES)["dark"]]: string };

function stepsPalette(theme: Theme): StepsPalette {
  const base: StepsPalette = STEPS_PALETTES[theme.mode];
  const picked = Object.entries(base).map(([key, value]) => [key, theme.colors[key] ?? value]);
  return Object.fromEntries(picked) as StepsPalette;
}

export type StepsLook = {
  palette: StepsPalette;
  font: string;
  textSize: number;
  glow: number;
  eyebrowCaps: boolean;
};

export function stepsLook(settings: ElementSettings["steps-timeline"]): StepsLook {
  return {
    palette: stepsPalette(settings.theme),
    font: fontOr(settings.fontFamily, "main"),
    textSize: settings.textSize,
    glow: settings.glow,
    eyebrowCaps: settings.eyebrowCaps,
  };
}
