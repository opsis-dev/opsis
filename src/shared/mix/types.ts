import { z } from "zod";

export const EQ_TYPES = ["lowshelf", "peak", "highshelf"] as const;
export type EqType = (typeof EQ_TYPES)[number];

const eqBandSchema = z.object({
  type: z.enum(EQ_TYPES),
  freq: z.number().min(20).max(20_000),
  gain: z.number().min(-18).max(18),
  q: z.number().min(0.1).max(10),
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

const DEFAULT_EQ_BANDS = bands([120, 0, 0.7], [500, 0, 1], [2500, 0, 1], [8000, 0, 0.7]);

const eqSchema = z.object({
  on: z.boolean().default(false),
  bands: z.array(eqBandSchema).length(4).default(DEFAULT_EQ_BANDS),
});
export type Eq = z.infer<typeof eqSchema>;

const compSchema = z.object({
  on: z.boolean().default(false),
  threshold: z.number().min(-60).max(0).default(-18),
  ratio: z.number().min(1).max(20).default(3),
  attack: z.number().min(0.1).max(500).default(10),
  release: z.number().min(5).max(3000).default(150),
  makeup: z.number().min(0).max(24).default(0),
});
export type Comp = z.infer<typeof compSchema>;

const highpassSchema = z.object({
  on: z.boolean().default(false),
  freq: z.number().min(20).max(1000).default(80),
});

export const MIN_DB = -60;
export const MAX_DB = 12;

export const busSettingsSchema = z.object({
  highpass: highpassSchema.prefault({}),
  eq: eqSchema.prefault({}),
  comp: compSchema.prefault({}),
  fader: z.number().min(MIN_DB).max(MAX_DB).default(0),
});
export type BusSettings = z.infer<typeof busSettingsSchema>;

export const stripSettingsSchema = z.object({
  trim: z.number().min(-24).max(24).default(0),
  ...busSettingsSchema.shape,
  pan: z.number().min(-1).max(1).default(0),
});
export type StripSettings = z.infer<typeof stripSettingsSchema>;

export const stripSchema = stripSettingsSchema.extend({
  mute: z.boolean().default(false),
  solo: z.boolean().default(false),
  bus: z.string().optional(),
});
export type Strip = z.infer<typeof stripSchema>;

const duckSettingsSchema = z.object({
  on: z.boolean().default(false),
  keys: z.array(z.string()).default([]),
  amount: z.number().min(1).max(24).default(10),
  attack: z.number().min(5).max(2000).default(80),
  release: z.number().min(20).max(5000).default(400),
  ignoreShorterMs: z.number().min(0).max(5000).default(0),
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

const GENTLE_MASTER_COMP = { threshold: -12, ratio: 2, attack: 20, release: 250 };

const masterSchema = z.object({
  fader: z.number().min(MIN_DB).max(MAX_DB).default(0),
  eq: eqSchema.prefault({}),
  comp: compSchema.prefault(GENTLE_MASTER_COMP),
  limiter: z
    .object({
      on: z.boolean().default(false),
      ceiling: z.number().min(-12).max(0).default(-1),
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

export type ChannelKind = "music" | "effect" | "clip";

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
