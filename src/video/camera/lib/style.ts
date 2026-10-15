import type { ReelLook } from "./look";
import type { PushAxis } from "./push";

export const STYLES = ["classic", "proof", "ledger", "stage"] as const;
export type ReelStyle = (typeof STYLES)[number];

export const DEFAULT_STYLE: ReelStyle = "proof";

export type MotionLayer = "cards" | "shots" | "both" | "type";

export type WordCadence = {
  fadeS: number;
} & (
  | { kind: "fixed"; staggerS: number }
  | { kind: "fitted"; staggerS: number; minStaggerS: number }
  | { kind: "typed"; perCharS: number }
);

export type ShotMove =
  | { kind: "push"; axis: PushAxis; dist: number; frames: number }
  | { kind: "ramp"; scale: number; durationS: number }
  | { kind: "none" };

export type CardLength = {
  holdS: number;
  holdFrom: "settled" | "lastWord";
  minS: number | null;
  maxS: number | null;
  trimInS: number;
};

export type CardStyle = {
  cadence: WordCadence;
  length: CardLength;
  enter: ShotMove;
  exit: ShotMove;
  emphasis: boolean;
};

export type ShotStyle = {
  framing: "window" | "fullbleed" | "isolate";
  chrome: boolean;
  windowFit: number | null;
  ground: string | null;
  zoom: boolean;
  cursor: boolean;
  ripple: boolean;
  enter: ShotMove;
  exit: ShotMove;
};

export type ChipStyle = {
  punchScale: number;
  punchS: number;
  leadS: number;
  settleS: number;
  afterPressS: number;
};

export type MarkSolo = {
  scale: number;
  growS: number;
  holdS: number;
  settleS: number;
  shiftFrac: number;
};

export type BookendStyle = {
  markSolo: MarkSolo | null;
  tumbleS: number;
  turns: number;
  driftPxPerFrame: number;
  driftFrames: number;
  minS: number | null;
};

export type GroundKey = "plate" | "plain" | "light";
export type Ground = { ground: string; ink: string; muted: string };

export type PaletteStyle = Record<GroundKey, Ground>;

export type TypeStyle = {
  sizePx: number;
  lineHeight: number;
  letterSpacing: string;
  capPx: number;
  pitchPx: number;
  fontFamily: string;
};

export type AnnotationStyle = {
  fill: string;
  ink: string;
  heightFrac: number;
  capRatio: number;
  radiusFrac: number;
  wipeS: number;
  oversize: number;
  exitS: number;
};

export type RecapStyle = {
  leadS: number;
  lockupStaggerS: number;
  itemsLeadS: number;
  itemStaggerS: number;
};

export type StyleSource = {
  file: string;
  shots: number;
  durationS: number;
  loudnessLUFS: number | null;
};

export type GrammarTargets = {
  meanShotS: number;
  cutsPerMin: number;
  movingFrac: number;
  longestStillF: number;
  cutDelta: "slam" | "matched";
};

export type ShotJoin = { kind: "cut" } | { kind: "dissolve"; frames: number };

export type HudStyle = { kind: "none" } | { kind: "steps" };

export type StylePreset = {
  look: ReelLook;
  motionLayer: MotionLayer;
  card: CardStyle;
  shot: ShotStyle;
  join: ShotJoin;
  hud: HudStyle;
  chip: ChipStyle;
  palette: PaletteStyle;
  type: TypeStyle;
  annotation: AnnotationStyle | null;
  bookend: BookendStyle;
  recap: RecapStyle;
  source: StyleSource | null;
  targets: GrammarTargets | null;
};

export const FONT_STACK_LEGACY = 'ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif';

export const INTER_STACK = `"Inter", ${FONT_STACK_LEGACY}`;

export const STYLE_PRESETS: Record<ReelStyle, StylePreset> = {
  classic: {
    look: "framed",
    motionLayer: "both",
    join: { kind: "cut" },
    hud: { kind: "none" },
    card: {
      cadence: { kind: "fixed", staggerS: 0.16, fadeS: 0 },
      length: {
        holdS: 1.2,
        holdFrom: "settled",
        minS: null,
        maxS: null,
        trimInS: 0,
      },
      enter: { kind: "none" },
      exit: { kind: "ramp", scale: 0.04, durationS: 0.35 },
      emphasis: true,
    },
    shot: {
      framing: "window",
      chrome: true,
      windowFit: 0.86,
      ground: null,
      zoom: true,
      cursor: true,
      ripple: true,
      enter: { kind: "none" },
      exit: { kind: "none" },
    },
    chip: {
      punchScale: 4,
      punchS: 0.45,
      leadS: 0.18,
      settleS: 0.03,
      afterPressS: 0.3,
    },
    type: {
      sizePx: 96,
      lineHeight: 1.12,
      letterSpacing: "-0.022em",
      capPx: 69,
      pitchPx: 108,
      fontFamily: FONT_STACK_LEGACY,
    },
    palette: {
      plate: { ground: "#08080a", ink: "#ffffff", muted: "rgba(255,255,255,0.62)" },
      plain: { ground: "#08080a", ink: "#ffffff", muted: "rgba(255,255,255,0.62)" },
      light: { ground: "#f4f2ec", ink: "#101317", muted: "rgba(16,19,23,0.58)" },
    },
    annotation: null,
    bookend: {
      markSolo: null,
      tumbleS: 0.85,
      turns: 1,
      driftPxPerFrame: 2,
      driftFrames: 14,
      minS: null,
    },
    recap: {
      leadS: 0.17,
      lockupStaggerS: 0.27,
      itemsLeadS: 0.37,
      itemStaggerS: 0.533,
    },
    source: null,
    targets: null,
  },

  proof: {
    look: "fullbleed",
    motionLayer: "cards",
    join: { kind: "cut" },
    hud: { kind: "none" },
    card: {
      cadence: { kind: "fitted", staggerS: 0.16, minStaggerS: 0.1, fadeS: 0 },
      length: {
        holdS: 2.07,
        holdFrom: "lastWord",
        minS: 3.2,
        maxS: 3.3,
        trimInS: 0.17,
      },
      enter: { kind: "push", axis: "y", dist: 56, frames: 14 },
      exit: { kind: "push", axis: "x", dist: -72, frames: 13 },
      emphasis: false,
    },
    shot: {
      framing: "fullbleed",
      chrome: false,
      windowFit: null,
      ground: null,
      zoom: true,
      cursor: true,
      ripple: false,
      enter: { kind: "push", axis: "x", dist: 114, frames: 15 },
      exit: { kind: "push", axis: "x", dist: 72, frames: 13 },
    },
    chip: {
      punchScale: 4,
      punchS: 0.45,
      leadS: 0.18,
      settleS: 0.03,
      afterPressS: 0.3,
    },
    type: {
      sizePx: 72,
      lineHeight: 1.194,
      letterSpacing: "0em",
      capPx: 52,
      pitchPx: 86,
      fontFamily: FONT_STACK_LEGACY,
    },
    palette: {
      plate: { ground: "#08080a", ink: "#e9ebe6", muted: "#8a8a86" },
      plain: { ground: "#08080a", ink: "#e9ebe6", muted: "#8a8a86" },
      light: { ground: "#edece5", ink: "#0a0a0a", muted: "#86857e" },
    },
    annotation: null,
    bookend: {
      markSolo: null,
      tumbleS: 0.85,
      turns: 1,
      driftPxPerFrame: 2,
      driftFrames: 14,
      minS: 3.2,
    },
    recap: {
      leadS: 0.17,
      lockupStaggerS: 0.27,
      itemsLeadS: 0.37,
      itemStaggerS: 0.533,
    },
    source: {
      file: "cursor-agent-ux-imrpovments-intro.mp4",
      shots: 12,
      durationS: 43.87,
      loudnessLUFS: -31.3,
    },
    targets: {
      meanShotS: 3.66,
      cutsPerMin: 15.0,
      movingFrac: 0.241,
      longestStillF: 110,
      cutDelta: "slam",
    },
  },

  ledger: {
    look: "fullbleed",
    motionLayer: "shots",
    join: { kind: "dissolve", frames: 6 },
    hud: { kind: "none" },
    card: {
      cadence: { kind: "fixed", staggerS: 0.2, fadeS: 0 },
      length: {
        holdS: 1.16,
        holdFrom: "lastWord",
        minS: null,
        maxS: null,
        trimInS: 0,
      },
      enter: { kind: "none" },
      exit: { kind: "none" },
      emphasis: false,
    },
    shot: {
      framing: "isolate",
      chrome: false,
      windowFit: null,
      ground: null,
      zoom: true,
      cursor: true,
      ripple: false,
      enter: { kind: "none" },
      exit: { kind: "none" },
    },
    chip: {
      punchScale: 4,
      punchS: 0.1,
      leadS: 0.03,
      settleS: 0.03,
      afterPressS: 0.03,
    },
    type: {
      sizePx: 98,
      lineHeight: 1.24,
      letterSpacing: "-0.01em",
      capPx: 70,
      pitchPx: 122,
      fontFamily: INTER_STACK,
    },
    palette: {
      plate: { ground: "#0a0a0a", ink: "#f0f05a", muted: "rgba(240,240,90,0.62)" },
      plain: { ground: "#f7fbf3", ink: "#0a0a0a", muted: "rgba(10,10,10,0.55)" },
      light: { ground: "#f7fbf3", ink: "#0a0a0a", muted: "rgba(10,10,10,0.55)" },
    },
    annotation: null,
    bookend: {
      markSolo: null,
      tumbleS: 0.85,
      turns: 1,
      driftPxPerFrame: 2,
      driftFrames: 14,
      minS: 2.63,
    },
    recap: {
      leadS: 0.17,
      lockupStaggerS: 0.27,
      itemsLeadS: 0.37,
      itemStaggerS: 0.533,
    },
    source: {
      file: "monid-claude-for-prospecting.mp4",
      shots: 5,
      durationS: 34.633,
      loudnessLUFS: -14.5,
    },
    targets: {
      meanShotS: 6.93,
      cutsPerMin: 6.9,
      movingFrac: 0.261,
      longestStillF: 77,
      cutDelta: "matched",
    },
  },
  stage: {
    look: "fullbleed",
    motionLayer: "shots",
    join: { kind: "cut" },
    hud: { kind: "none" },
    card: {
      cadence: { kind: "typed", perCharS: 0.0833, fadeS: 0 },
      length: {
        holdS: 0.6,
        holdFrom: "lastWord",
        minS: null,
        maxS: null,
        trimInS: 0,
      },
      enter: { kind: "none" },
      exit: { kind: "none" },
      emphasis: true,
    },
    shot: {
      framing: "window",
      chrome: false,
      windowFit: 0.84,
      ground: "#faf6f1",
      zoom: false,
      cursor: true,
      ripple: false,
      enter: { kind: "push", axis: "scale", dist: -0.2, frames: 11 },
      exit: { kind: "push", axis: "y", dist: 1520, frames: 11 },
    },
    chip: {
      punchScale: 4,
      punchS: 0.1,
      leadS: 0.03,
      settleS: 0.03,
      afterPressS: 0.03,
    },
    type: {
      sizePx: 82,
      lineHeight: 1.2,
      letterSpacing: "-0.01em",
      capPx: 59,
      pitchPx: 98,
      fontFamily: INTER_STACK,
    },
    palette: {
      plate: { ground: "#faf6f1", ink: "#0a0a0a", muted: "rgba(10,10,10,0.55)" },
      plain: { ground: "#faf6f1", ink: "#0a0a0a", muted: "rgba(10,10,10,0.55)" },
      light: { ground: "#faf6f1", ink: "#0a0a0a", muted: "rgba(10,10,10,0.55)" },
    },
    annotation: {
      fill: "#f0f05a",
      ink: "#0a0a0a",
      heightFrac: 0.073,
      capRatio: 0.58,
      radiusFrac: 0.18,
      wipeS: 0.208,
      oversize: 1.06,
      exitS: 0.16,
    },
    bookend: {
      markSolo: {
        scale: 3.7,
        growS: 0.375,
        holdS: 0.58,
        settleS: 0.71,
        shiftFrac: 0.42,
      },
      tumbleS: 0.85,
      turns: 1,
      driftPxPerFrame: 2,
      driftFrames: 14,
      minS: 3.0,
    },
    recap: {
      leadS: 0.17,
      lockupStaggerS: 0.27,
      itemsLeadS: 0.37,
      itemStaggerS: 0.533,
    },
    source: {
      file: "Replit Replit X.mp4",
      shots: 9,
      durationS: 42.048,
      loudnessLUFS: -14.0,
    },
    targets: {
      meanShotS: 4.67,
      cutsPerMin: 0,
      movingFrac: 0.678,
      longestStillF: 68,
      cutDelta: "matched",
    },
  },
};

export const REFERENCE_FILMS: {
  file: string;
  label: string;
  style: ReelStyle | null;
  durationS: number;
  shots: number;
  movingFrac: number;
  cutsPerMin: number;
  loudnessLUFS: number | null;
  note?: string;
}[] = [
  {
    file: "cursor-agent-ux-imrpovments-intro.mp4",
    label: "Cursor — Agent UX improvements",
    style: "proof",
    durationS: 43.87,
    shots: 12,
    movingFrac: 0.241,
    cutsPerMin: 15.0,
    loudnessLUFS: -31.3,
  },
  {
    file: "cursor_origin_intro.mp4",
    label: "Cursor — origin (Film B)",
    style: null,
    durationS: 30.9,
    shots: 17,
    movingFrac: 0.368,
    cutsPerMin: 31.1,
    loudnessLUFS: null,
  },
  {
    file: "monid-claude-for-prospecting.mp4",
    label: "monid — Claude for prospecting",
    style: "ledger",
    durationS: 34.633,
    shots: 5,
    movingFrac: 0.261,
    cutsPerMin: 6.9,
    loudnessLUFS: -14.5,
    note: "Delivered mislabelled as a Replit film. Its persistent HUD is unbuilt.",
  },
  {
    file: "uber-base-icon-system.mp4",
    label: "Uber — A new icon system",
    style: null,
    durationS: 40.768,
    shots: 11,
    movingFrac: 0.812,
    cutsPerMin: 14.7,
    loudnessLUFS: -8.2,
    note:
      "NOT IMPLEMENTABLE as a preset. Kinetic typography — glyphs transform " +
      "continuously along curved baselines, as outlines, as ghost stacks. " +
      "Its transferable findings (75px cap, exact #000/#FFF grounds, the " +
      'motionLayer "type" axis) are taken without it.',
  },
];

export function isStyle(value: unknown): value is ReelStyle {
  return typeof value === "string" && (STYLES as readonly string[]).includes(value);
}

export function resolveStyle(input: { style?: ReelStyle; look?: ReelLook }): ReelStyle {
  if (input.style) return input.style;
  if (input.look === "fullbleed") return "proof";
  if (input.look === "framed") return "classic";
  return DEFAULT_STYLE;
}

export function resolvePreset(input: { style?: ReelStyle; look?: ReelLook }): StylePreset {
  return STYLE_PRESETS[resolveStyle(input)];
}

export function styleProblem(input: { style?: unknown; look?: ReelLook }): string | null {
  const { style, look } = input;
  if (style === undefined) return null;
  if (!isStyle(style))
    return `\`style\` must be one of ${STYLES.join(", ")} (got ${JSON.stringify(style)})`;
  if (style === "classic" && look === "fullbleed")
    return '`style: "classic"` cannot be combined with `look: "fullbleed"` — classic IS the framed grammar';
  return null;
}
