import { describe, expect, it } from "vitest";
import {
  addBus,
  applyBusSettings,
  removeBus,
  resetBus,
  resetStrip,
  stripSettings,
  updateChannels,
} from "./edits";
import { DEFAULT_STRIP, STRIP_PRESETS, busSchema, defaultMix } from "./types";

describe("mix edits", () => {
  it("copies a strip's settings without its routing", () => {
    const strip = { ...DEFAULT_STRIP, trim: 3, pan: -0.5, mute: true, solo: true, bus: "music" };
    expect(stripSettings(strip)).toEqual({ ...stripSettings(DEFAULT_STRIP), trim: 3, pan: -0.5 });
  });

  it("resets a strip but keeps its routing", () => {
    const strip = { ...DEFAULT_STRIP, trim: 3, fader: -6, mute: true, bus: "music" };
    expect(resetStrip(strip)).toEqual({ ...DEFAULT_STRIP, mute: true, bus: "music" });
  });

  it("applies the bus part of a preset and resets it", () => {
    const bus = busSchema.parse({ id: "b", name: "Voices", mute: true });
    const preset = STRIP_PRESETS[0].strip;
    const applied = applyBusSettings(bus, { ...preset, trim: 6, fader: -3 });
    expect(applied).toMatchObject({ id: "b", name: "Voices", mute: true, fader: -3 });
    expect(applied.comp).toEqual(preset.comp);
    expect(applied).not.toHaveProperty("trim");
    expect(resetBus(applied)).toEqual(bus);
  });

  it("adds and removes a bus everywhere it is used", () => {
    const base = addBus(defaultMix(), "extra");
    const keyed = {
      ...base,
      buses: base.buses.map((b) =>
        b.id === "music" ? { ...b, duck: { ...b.duck, keys: ["extra"] } } : b,
      ),
    };
    const routed = updateChannels(keyed, ["a"], (s) => ({ ...s, bus: "extra" }));
    const removed = removeBus(routed, "extra");
    expect(removed.buses.map((b) => b.id)).toEqual(["music", "effects", "clips"]);
    expect(removed.buses[0].duck.keys).toEqual([]);
    expect(removed.channels.a.bus).toBeUndefined();
  });
});
