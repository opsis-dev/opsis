import { z } from "zod";

export const SAMPLE_RATE = 48_000;

type Bounds = { min: number; max: number };

export type NumberRange = Bounds & { default: number };

const bounded = ({ min, max }: Bounds) => z.number().min(min).max(max);

const ranged = (range: NumberRange) => bounded(range).default(range.default);

export const EQ_TYPES = ["lowshelf", "peak", "highshelf"] as const;
export type EqType = (typeof EQ_TYPES)[number];

export const EQ_BAND_BOUNDS = {
  freq: { min: 20, max: 20_000 },
  gain: { min: -18, max: 18 },
  q: { min: 0.1, max: 10 },
} as const satisfies Record<string, Bounds>;

const eqBandSchema = z.object({
  type: z.enum(EQ_TYPES),
  freq: bounded(EQ_BAND_BOUNDS.freq),
  gain: bounded(EQ_BAND_BOUNDS.gain),
  q: bounded(EQ_BAND_BOUNDS.q),
});
export type EqBand = z.infer<typeof eqBandSchema>;

type BandShape = [freq: number, gain: number, q: number];

const band = (type: EqType, [freq, gain, q]: BandShape): EqBand => ({
  type,
  freq,
  gain,
  q,
});

const bands = (low: BandShape, lowMid: BandShape, highMid: BandShape, high: BandShape) => [
  band("lowshelf", low),
  band("peak", lowMid),
  band("peak", highMid),
  band("highshelf", high),
];

export const DEFAULT_EQ_BANDS = bands([120, 0, 0.7], [500, 0, 1], [2500, 0, 1], [8000, 0, 0.7]);

const eqSchema = z.object({
  on: z.boolean().default(false),
  bands: z.array(eqBandSchema).length(4).default(DEFAULT_EQ_BANDS),
});
export type Eq = z.infer<typeof eqSchema>;

export const COMP_RANGES = {
  threshold: { min: -60, max: 0, default: -18 },
  ratio: { min: 1, max: 20, default: 3 },
  attack: { min: 0.1, max: 500, default: 10 },
  release: { min: 5, max: 3000, default: 150 },
  makeup: { min: 0, max: 24, default: 0 },
} as const satisfies Record<string, NumberRange>;

const compSchema = z.object({
  on: z.boolean().default(false),
  threshold: ranged(COMP_RANGES.threshold),
  ratio: ranged(COMP_RANGES.ratio),
  attack: ranged(COMP_RANGES.attack),
  release: ranged(COMP_RANGES.release),
  makeup: ranged(COMP_RANGES.makeup),
});
export type Comp = z.infer<typeof compSchema>;

export const HIGHPASS_RANGE: NumberRange = { min: 20, max: 1000, default: 80 };

const highpassSchema = z.object({
  on: z.boolean().default(false),
  freq: ranged(HIGHPASS_RANGE),
});

export const MIN_DB = -60;
export const SILENCE_DB = -120;
export const MAX_DB = 12;

const FADER_RANGE: NumberRange = { min: MIN_DB, max: MAX_DB, default: 0 };

const busSettingsSchema = z.object({
  highpass: highpassSchema.prefault({}),
  eq: eqSchema.prefault({}),
  comp: compSchema.prefault({}),
  fader: ranged(FADER_RANGE),
});
export type BusSettings = z.infer<typeof busSettingsSchema>;

export const TRIM_RANGE: NumberRange = { min: -24, max: 24, default: 0 };
export const PAN_RANGE: NumberRange = { min: -1, max: 1, default: 0 };

const stripSettingsSchema = z.object({
  trim: ranged(TRIM_RANGE),
  ...busSettingsSchema.shape,
  pan: ranged(PAN_RANGE),
});
export type StripSettings = z.infer<typeof stripSettingsSchema>;

const stripSchema = stripSettingsSchema.extend({
  mute: z.boolean().default(false),
  solo: z.boolean().default(false),
  bus: z.string().optional(),
});
export type Strip = z.infer<typeof stripSchema>;

export const DUCK_RANGES = {
  amount: { min: 1, max: 24, default: 10 },
  attack: { min: 5, max: 2000, default: 80 },
  release: { min: 20, max: 5000, default: 400 },
  ignoreShorterMs: { min: 0, max: 5000, default: 0 },
} as const satisfies Record<string, NumberRange>;

const duckSettingsSchema = z.object({
  on: z.boolean().default(false),
  keys: z.array(z.string()).default([]),
  amount: ranged(DUCK_RANGES.amount),
  attack: ranged(DUCK_RANGES.attack),
  release: ranged(DUCK_RANGES.release),
  ignoreShorterMs: ranged(DUCK_RANGES.ignoreShorterMs),
});
export type DuckSettings = z.infer<typeof duckSettingsSchema>;

export const busSchema = z.object({
  id: z.string().min(1),
  name: z.string(),
  ...busSettingsSchema.shape,
  mute: z.boolean().default(false),
  solo: z.boolean().default(false),
  duck: duckSettingsSchema.prefault({}),
});
export type Bus = z.infer<typeof busSchema>;

export const LOUDNESS_CHOICES = ["off", "web", "podcast", "broadcast"] as const;
export type LoudnessTarget = (typeof LOUDNESS_CHOICES)[number];

export const LOUDNESS_TARGETS = {
  off: null,
  web: { lufs: -14, truePeak: -1 },
  podcast: { lufs: -16, truePeak: -1 },
  broadcast: { lufs: -23, truePeak: -2 },
} as const satisfies Record<LoudnessTarget, { lufs: number; truePeak: number } | null>;

export const LIMITER_CEILING_RANGE: NumberRange = { min: -12, max: 0, default: -1 };

const GENTLE_MASTER_COMP = { threshold: -12, ratio: 2, attack: 20, release: 250 };

const masterSchema = z.object({
  fader: ranged(FADER_RANGE),
  eq: eqSchema.prefault({}),
  comp: compSchema.prefault(GENTLE_MASTER_COMP),
  limiter: z
    .object({
      on: z.boolean().default(false),
      ceiling: ranged(LIMITER_CEILING_RANGE),
    })
    .prefault({}),
  loudness: z.enum(LOUDNESS_CHOICES).default("off"),
});
export type Master = z.infer<typeof masterSchema>;

const DEFAULT_BUSES: Bus[] = [
  busSchema.parse({ id: "music", name: "Music" }),
  busSchema.parse({ id: "effects", name: "Effects" }),
  busSchema.parse({ id: "clips", name: "Clips" }),
];

export const BUILT_IN_BUSES = new Set(DEFAULT_BUSES.map((b) => b.id));

export const mixSchema = z.object({
  channels: z.record(z.string(), stripSchema).default({}),
  buses: z.array(busSchema).min(1).default(DEFAULT_BUSES),
  master: masterSchema.prefault({}),
});
export type Mix = z.infer<typeof mixSchema>;

export const DEFAULT_STRIP: Strip = stripSchema.parse({});
export const DEFAULT_MASTER: Master = masterSchema.parse({});
export const defaultMix = (): Mix => mixSchema.parse({});

export const CHANNEL_KINDS = ["music", "effect", "clip"] as const;
export type ChannelKind = (typeof CHANNEL_KINDS)[number];

export const DEFAULT_BUS_FOR: Record<ChannelKind, string> = {
  music: "music",
  effect: "effects",
  clip: "clips",
};

type StripPreset = { id: string; label: string; strip: Partial<StripSettings> };

const eqOn = (...shape: Parameters<typeof bands>): Eq => ({ on: true, bands: bands(...shape) });

export const STRIP_PRESETS: StripPreset[] = [
  {
    id: "voice",
    label: "Voice",
    strip: {
      highpass: { on: true, freq: 90 },
      eq: eqOn([150, -2, 0.7], [350, -2, 1.2], [3000, 3, 1], [10_000, 2, 0.7]),
      comp: { on: true, threshold: -20, ratio: 3, attack: 8, release: 120, makeup: 3 },
    },
  },
  {
    id: "music-bed",
    label: "Music bed",
    strip: {
      highpass: { on: true, freq: 40 },
      eq: eqOn([120, 0, 0.7], [400, 0, 1], [2500, -3, 1.4], [9000, -1, 0.7]),
      comp: { on: true, threshold: -24, ratio: 2, attack: 30, release: 300, makeup: 0 },
      fader: -6,
    },
  },
  {
    id: "punchy-effects",
    label: "Punchy effects",
    strip: {
      highpass: { on: true, freq: 60 },
      eq: eqOn([100, 2, 0.7], [600, -2, 1], [4000, 2, 1], [9000, 1, 0.7]),
      comp: { on: true, threshold: -16, ratio: 4, attack: 3, release: 80, makeup: 2 },
    },
  },
];

export const dbToGain = (db: number) => (db <= MIN_DB ? 0 : 10 ** (db / 20));
