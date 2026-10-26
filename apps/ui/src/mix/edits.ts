import { pick } from "@opsis/video/common/objects";
import {
  BUS_SETTING_KEYS,
  DEFAULT_STRIP,
  newBus,
  stripOf,
  stripSettings,
  type BusSettings,
  type StripSettings,
} from "@/core/mix";
import type { Bus, Mix, Strip } from "@/generated/bindings";

const busSettings = (settings: BusSettings): BusSettings => pick(settings, BUS_SETTING_KEYS);

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
  buses: [...mix.buses, newBus(id, `Bus ${mix.buses.length + 1}`)],
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
