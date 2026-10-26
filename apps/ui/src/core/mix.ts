import { findById } from "@opsis/video/common/lists";
import type { MinMax } from "@opsis/video/common/math";
import type {
  Bus,
  ChannelKind,
  Comp,
  DuckSettings,
  Eq,
  EqBand,
  EqType,
  Highpass,
  LoudnessTarget,
  Master,
  Mix,
  Strip,
} from "@/generated/bindings";

export const SAMPLE_RATE = 48_000;

export type NumberRange = MinMax & { default: number };

export const EQ_TYPES: EqType[] = ["lowshelf", "peak", "highshelf"];

export const EQ_BAND_BOUNDS = {
  freq: { min: 20, max: 20_000 },
  gain: { min: -18, max: 18 },
  q: { min: 0.1, max: 10 },
} as const satisfies Record<string, MinMax>;

type BandShape = [freq: number, gain: number, q: number];

const band = (type: EqType, [freq, gain, q]: BandShape): EqBand => ({ type, freq, gain, q });

const bands = (low: BandShape, lowMid: BandShape, highMid: BandShape, high: BandShape) => [
  band("lowshelf", low),
  band("peak", lowMid),
  band("peak", highMid),
  band("highshelf", high),
];

export const DEFAULT_EQ_BANDS = bands([120, 0, 0.7], [500, 0, 1], [2500, 0, 1], [8000, 0, 0.7]);

export const COMP_RANGES = {
  threshold: { min: -60, max: 0, default: -18 },
  ratio: { min: 1, max: 20, default: 3 },
  attack: { min: 0.1, max: 500, default: 10 },
  release: { min: 5, max: 3000, default: 150 },
  makeup: { min: 0, max: 24, default: 0 },
} as const satisfies Record<string, NumberRange>;

export const HIGHPASS_RANGE: NumberRange = { min: 20, max: 1000, default: 80 };

export const MIN_DB = -60;
export const SILENCE_DB = -120;
export const MAX_DB = 12;

export const TRIM_RANGE: NumberRange = { min: -24, max: 24, default: 0 };
export const PAN_RANGE: NumberRange = { min: -1, max: 1, default: 0 };

export const DUCK_RANGES = {
  amount: { min: 1, max: 24, default: 10 },
  attack: { min: 5, max: 2000, default: 80 },
  release: { min: 20, max: 5000, default: 400 },
  ignoreShorterMs: { min: 0, max: 5000, default: 0 },
} as const satisfies Record<string, NumberRange>;

export const LOUDNESS_CHOICES: LoudnessTarget[] = ["off", "web", "podcast", "broadcast"];

export const LOUDNESS_TARGETS = {
  off: null,
  web: { lufs: -14, truePeak: -1 },
  podcast: { lufs: -16, truePeak: -1 },
  broadcast: { lufs: -23, truePeak: -2 },
} as const satisfies Record<LoudnessTarget, { lufs: number; truePeak: number } | null>;

export const LIMITER_CEILING_RANGE: NumberRange = { min: -12, max: 0, default: -1 };

const eqOff = (): Eq => ({ on: false, bands: DEFAULT_EQ_BANDS.map((b) => ({ ...b })) });

const comp = (shape: Partial<Comp> = {}): Comp => ({
  on: false,
  threshold: COMP_RANGES.threshold.default,
  ratio: COMP_RANGES.ratio.default,
  attack: COMP_RANGES.attack.default,
  release: COMP_RANGES.release.default,
  makeup: COMP_RANGES.makeup.default,
  ...shape,
});

const highpassOff = (): Highpass => ({ on: false, freq: HIGHPASS_RANGE.default });

export type BusSettings = Pick<Bus, "highpass" | "eq" | "comp" | "fader">;

export type StripSettings = Pick<Strip, "trim" | "highpass" | "eq" | "comp" | "fader" | "pan">;

export const BUS_SETTING_KEYS: (keyof BusSettings)[] = ["highpass", "eq", "comp", "fader"];

const STRIP_SETTING_KEYS: (keyof StripSettings)[] = ["trim", ...BUS_SETTING_KEYS, "pan"];

const busSettingsDefaults = (): BusSettings => ({
  highpass: highpassOff(),
  eq: eqOff(),
  comp: comp(),
  fader: 0,
});

export const DEFAULT_STRIP: Strip = {
  trim: TRIM_RANGE.default,
  ...busSettingsDefaults(),
  pan: PAN_RANGE.default,
  mute: false,
  solo: false,
};

export const stripSettings = (strip: StripSettings): StripSettings =>
  Object.fromEntries(STRIP_SETTING_KEYS.map((k) => [k, strip[k]])) as StripSettings;

const duckOff = (): DuckSettings => ({
  on: false,
  keys: [],
  amount: DUCK_RANGES.amount.default,
  attack: DUCK_RANGES.attack.default,
  release: DUCK_RANGES.release.default,
  ignoreShorterMs: DUCK_RANGES.ignoreShorterMs.default,
});

export const newBus = (id: string, name: string): Bus => ({
  id,
  name,
  ...busSettingsDefaults(),
  mute: false,
  solo: false,
  duck: duckOff(),
});

export const DEFAULT_MASTER: Master = {
  fader: 0,
  eq: eqOff(),
  comp: comp({ threshold: -12, ratio: 2, attack: 20, release: 250 }),
  limiter: { on: false, ceiling: LIMITER_CEILING_RANGE.default },
  loudness: "off",
};

const BUS_IDS = { music: "music", effects: "effects", clips: "clips" } as const;

export const BUILT_IN_BUSES: ReadonlySet<string> = new Set(Object.values(BUS_IDS));

export const defaultMix = (): Mix => ({
  channels: {},
  buses: [
    newBus(BUS_IDS.music, "Music"),
    newBus(BUS_IDS.effects, "Effects"),
    newBus(BUS_IDS.clips, "Clips"),
  ],
  master: structuredClone(DEFAULT_MASTER),
});

const DEFAULT_BUS_FOR: Record<ChannelKind, string> = {
  music: BUS_IDS.music,
  effect: BUS_IDS.effects,
  clip: BUS_IDS.clips,
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

export const resolveMix = (mix: Mix | undefined | null): Mix => mix ?? defaultMix();

export const stripOf = (mix: Mix, channel: string): Strip => mix.channels[channel] ?? DEFAULT_STRIP;

export function busOf(mix: Mix, channel: string, kind: ChannelKind): Bus {
  const strip = stripOf(mix, channel);
  return (
    findById(mix.buses, strip.bus) ?? findById(mix.buses, DEFAULT_BUS_FOR[kind]) ?? mix.buses[0]
  );
}

export function soloActive(mix: Mix): boolean {
  return Object.values(mix.channels).some((s) => s.solo) || mix.buses.some((b) => b.solo);
}

export type Audibility = (channel: string, kind: ChannelKind) => boolean;

export function audibility(mix: Mix): Audibility {
  const solo = soloActive(mix);
  return (channel, kind) => {
    const strip = stripOf(mix, channel);
    const bus = busOf(mix, channel, kind);
    if (strip.mute || bus.mute) return false;
    return !solo || strip.solo || bus.solo;
  };
}
