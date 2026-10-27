import type { FieldWidget } from "@opsis/video/generated/spec";
type FieldPreset = { label: string; values: Record<string, unknown> };

export const THEME_MODES = ["light", "dark"] as const;

type ThemeMode = (typeof THEME_MODES)[number];

export type FieldMeta = {
  description?: string;
  help?: string;
  widget?: FieldWidget;
  step?: number;
  unit?: string;
  accept?: string;
  advanced?: boolean;
  hidden?: boolean;
  labels?: Partial<Record<string, string>>;
  zeroLabel?: string;
  unconnectedZeroLabel?: string;
  presets?: FieldPreset[];
  presetsLabel?: string;
  pauses?: boolean;
  timeKey?: string;
  lengthKey?: string;
  showIf?: Record<string, string[]>;
  showIfFile?: Record<string, Array<"image" | "video" | "audio">>;
  themes?: Partial<Record<ThemeMode, Record<string, string>>>;
  framesAt?: number;
};

export type ElementCategory =
  | "core"
  | "text"
  | "overlay"
  | "transition"
  | "background"
  | "effect"
  | "ui"
  | "data"
  | "scene"
  | "device";

export type ElementKind = "component" | "wrapper" | "presentation" | "sequencer";

export const META_KEYWORDS: Record<Exclude<keyof FieldMeta, "description">, string> = {
  help: "x-help",
  widget: "x-widget",
  step: "x-step",
  unit: "x-unit",
  accept: "x-accept",
  advanced: "x-advanced",
  hidden: "x-hidden",
  labels: "x-labels",
  zeroLabel: "x-zero-label",
  unconnectedZeroLabel: "x-unconnected-zero-label",
  presets: "x-presets",
  presetsLabel: "x-presets-label",
  pauses: "x-pauses",
  timeKey: "x-time-key",
  lengthKey: "x-length-key",
  showIf: "x-show-if",
  showIfFile: "x-show-if-file",
  themes: "x-themes",
  framesAt: "x-frames-at",
};
