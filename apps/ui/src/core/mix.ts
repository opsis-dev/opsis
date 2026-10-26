import { pick } from "@opsis/video/common/objects";
import type { Bus, Strip } from "@/generated/bindings";
import { mixerBook } from "@/store/catalog";

export type BusSettings = Pick<Bus, "highpass" | "eq" | "comp" | "fader">;

export type StripSettings = Pick<Strip, "trim" | "highpass" | "eq" | "comp" | "fader" | "pan">;

const BUS_SETTING_KEYS: (keyof BusSettings)[] = ["highpass", "eq", "comp", "fader"];

const STRIP_SETTING_KEYS: (keyof StripSettings)[] = ["trim", ...BUS_SETTING_KEYS, "pan"];

export const busSettings = (settings: Partial<StripSettings>): Partial<BusSettings> =>
  pick(settings, BUS_SETTING_KEYS);

export const stripSettings = (strip: StripSettings): StripSettings =>
  pick(strip, STRIP_SETTING_KEYS);

export const dbToGain = (db: number): number => (db <= mixerBook().minDb ? 0 : 10 ** (db / 20));
