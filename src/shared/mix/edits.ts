import { stripOf } from "./levels";
import {
  DEFAULT_STRIP,
  busSchema,
  type Bus,
  type BusSettings,
  type Mix,
  type Strip,
  type StripSettings,
} from "./types";

const busSettings = ({ highpass, eq, comp, fader }: BusSettings): BusSettings => ({
  highpass,
  eq,
  comp,
  fader,
});

export const stripSettings = (strip: StripSettings): StripSettings => ({
  trim: strip.trim,
  ...busSettings(strip),
  pan: strip.pan,
});

export const resetStrip = (strip: Strip): Strip => ({ ...strip, ...stripSettings(DEFAULT_STRIP) });

export const applyBusSettings = (bus: Bus, settings: Partial<StripSettings>): Bus => ({
  ...bus,
  ...busSettings({ ...bus, ...settings }),
});

export const resetBus = (bus: Bus): Bus => ({ ...bus, ...busSettings(DEFAULT_STRIP) });

export const updateChannels = (mix: Mix, ids: string[], fn: (s: Strip) => Strip): Mix => ({
  ...mix,
  channels: {
    ...mix.channels,
    ...Object.fromEntries(ids.map((id) => [id, fn(stripOf(mix, id))])),
  },
});

export const updateBus = (mix: Mix, id: string, fn: (b: Bus) => Bus): Mix => ({
  ...mix,
  buses: mix.buses.map((b) => (b.id === id ? fn(b) : b)),
});

export const addBus = (mix: Mix, id: string): Mix => ({
  ...mix,
  buses: [...mix.buses, busSchema.parse({ id, name: `Bus ${mix.buses.length + 1}` })],
});

export function removeBus(mix: Mix, id: string): Mix {
  return {
    ...mix,
    buses: mix.buses
      .filter((b) => b.id !== id)
      .map((b) => ({ ...b, duck: { ...b.duck, keys: b.duck.keys.filter((k) => k !== id) } })),
    channels: Object.fromEntries(
      Object.entries(mix.channels).map(([channel, s]) => [
        channel,
        s.bus === id ? { ...s, bus: undefined } : s,
      ]),
    ),
  };
}
