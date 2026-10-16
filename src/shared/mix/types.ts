import { z } from "zod";

export const EQ_TYPES = ["lowshelf", "peak", "highshelf"] as const;
export type EqType = (typeof EQ_TYPES)[number];

export const eqBandSchema = z.object({
  type: z.enum(EQ_TYPES),
  freq: z.number().min(20).max(20_000),
  gain: z.number().min(-18).max(18),
  q: z.number().min(0.1).max(10),
});
export type EqBand = z.infer<typeof eqBandSchema>;

export const DEFAULT_EQ_BANDS: EqBand[] = [
  { type: "lowshelf", freq: 120, gain: 0, q: 0.7 },
  { type: "peak", freq: 500, gain: 0, q: 1 },
  { type: "peak", freq: 2500, gain: 0, q: 1 },
  { type: "highshelf", freq: 8000, gain: 0, q: 0.7 },
];

export const eqSchema = z.object({
  on: z.boolean().default(false),
  bands: z.array(eqBandSchema).length(4).default(DEFAULT_EQ_BANDS),
});
export type Eq = z.infer<typeof eqSchema>;

export const compSchema = z.object({
  on: z.boolean().default(false),
  threshold: z.number().min(-60).max(0).default(-18),
  ratio: z.number().min(1).max(20).default(3),
  attack: z.number().min(0.1).max(500).default(10),
  release: z.number().min(5).max(3000).default(150),
  makeup: z.number().min(0).max(24).default(0),
});
export type Comp = z.infer<typeof compSchema>;

export const highpassSchema = z.object({
  on: z.boolean().default(false),
  freq: z.number().min(20).max(1000).default(80),
});

export const MIN_DB = -60;

const OFF_COMP = {
  on: false,
  threshold: -18,
  ratio: 3,
  attack: 10,
  release: 150,
  makeup: 0,
};

export const stripSchema = z.object({
  trim: z.number().min(-24).max(24).default(0),
  highpass: highpassSchema.default({ on: false, freq: 80 }),
  eq: eqSchema.default({ on: false, bands: DEFAULT_EQ_BANDS }),
  comp: compSchema.default(OFF_COMP),
  pan: z.number().min(-1).max(1).default(0),
  fader: z.number().min(MIN_DB).max(12).default(0),
  mute: z.boolean().default(false),
  solo: z.boolean().default(false),
  bus: z.string().optional(),
});
export type Strip = z.infer<typeof stripSchema>;

export const duckSettingsSchema = z.object({
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
  highpass: highpassSchema.default({ on: false, freq: 80 }),
  eq: eqSchema.default({ on: false, bands: DEFAULT_EQ_BANDS }),
  comp: compSchema.default(OFF_COMP),
  fader: z.number().min(MIN_DB).max(12).default(0),
  mute: z.boolean().default(false),
  solo: z.boolean().default(false),
  duck: duckSettingsSchema.default({
    on: false,
    keys: [],
    amount: 10,
    attack: 80,
    release: 400,
    ignoreShorterMs: 0,
  }),
});
export type Bus = z.infer<typeof busSchema>;

export const LOUDNESS_TARGETS = {
  off: null,
  web: { lufs: -14, truePeak: -1 },
  podcast: { lufs: -16, truePeak: -1 },
  broadcast: { lufs: -23, truePeak: -2 },
} as const;
export type LoudnessTarget = keyof typeof LOUDNESS_TARGETS;

export const masterSchema = z.object({
  fader: z.number().min(MIN_DB).max(12).default(0),
  eq: eqSchema.default({ on: false, bands: DEFAULT_EQ_BANDS }),
  comp: compSchema.default({
    on: false,
    threshold: -12,
    ratio: 2,
    attack: 20,
    release: 250,
    makeup: 0,
  }),
  limiter: z
    .object({
      on: z.boolean().default(false),
      ceiling: z.number().min(-12).max(0).default(-1),
    })
    .default({ on: false, ceiling: -1 }),
  loudness: z.enum(["off", "web", "podcast", "broadcast"]).default("off"),
});
export type Master = z.infer<typeof masterSchema>;

export const DEFAULT_BUSES: Bus[] = [
  busSchema.parse({ id: "music", name: "Music" }),
  busSchema.parse({ id: "effects", name: "Effects" }),
  busSchema.parse({ id: "clips", name: "Clips" }),
];

export const BUILT_IN_BUSES = new Set(DEFAULT_BUSES.map((b) => b.id));

export const mixSchema = z.object({
  channels: z.record(z.string(), stripSchema).default({}),
  buses: z.array(busSchema).default(DEFAULT_BUSES),
  master: masterSchema.default(masterSchema.parse({})),
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

export type StripPreset = { id: string; label: string; strip: Partial<Strip> };

export const STRIP_PRESETS: StripPreset[] = [
  {
    id: "voice",
    label: "Voice",
    strip: {
      highpass: { on: true, freq: 90 },
      eq: {
        on: true,
        bands: [
          { type: "lowshelf", freq: 150, gain: -2, q: 0.7 },
          { type: "peak", freq: 350, gain: -2, q: 1.2 },
          { type: "peak", freq: 3000, gain: 3, q: 1 },
          { type: "highshelf", freq: 10_000, gain: 2, q: 0.7 },
        ],
      },
      comp: { on: true, threshold: -20, ratio: 3, attack: 8, release: 120, makeup: 3 },
    },
  },
  {
    id: "music-bed",
    label: "Music bed",
    strip: {
      highpass: { on: true, freq: 40 },
      eq: {
        on: true,
        bands: [
          { type: "lowshelf", freq: 120, gain: 0, q: 0.7 },
          { type: "peak", freq: 400, gain: 0, q: 1 },
          { type: "peak", freq: 2500, gain: -3, q: 1.4 },
          { type: "highshelf", freq: 9000, gain: -1, q: 0.7 },
        ],
      },
      comp: { on: true, threshold: -24, ratio: 2, attack: 30, release: 300, makeup: 0 },
      fader: -6,
    },
  },
  {
    id: "punchy-effects",
    label: "Punchy effects",
    strip: {
      highpass: { on: true, freq: 60 },
      eq: {
        on: true,
        bands: [
          { type: "lowshelf", freq: 100, gain: 2, q: 0.7 },
          { type: "peak", freq: 600, gain: -2, q: 1 },
          { type: "peak", freq: 4000, gain: 2, q: 1 },
          { type: "highshelf", freq: 9000, gain: 1, q: 0.7 },
        ],
      },
      comp: { on: true, threshold: -16, ratio: 4, attack: 3, release: 80, makeup: 2 },
    },
  },
];

export const dbToGain = (db: number) => (db <= MIN_DB ? 0 : 10 ** (db / 20));
export const gainToDb = (gain: number) => (gain <= 0 ? -Infinity : 20 * Math.log10(gain));
