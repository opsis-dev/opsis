import { stripOf } from "./levels";
import {
  busSchema,
  busSettingsSchema,
  stripSettingsSchema,
  type Bus,
  type Mix,
  type Strip,
  type StripSettings,
} from "./types";

const DEFAULT_STRIP_SETTINGS = stripSettingsSchema.parse({});
const DEFAULT_BUS_SETTINGS = busSettingsSchema.parse({});

export const stripSettings = (strip: Strip): StripSettings => stripSettingsSchema.parse(strip);

export const resetStrip = (strip: Strip): Strip => ({ ...strip, ...DEFAULT_STRIP_SETTINGS });

export const applyBusSettings = (bus: Bus, settings: Partial<StripSettings>): Bus => ({
  ...bus,
  ...busSettingsSchema.parse({ ...bus, ...settings }),
});

export const resetBus = (bus: Bus): Bus => ({ ...bus, ...DEFAULT_BUS_SETTINGS });

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
