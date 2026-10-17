import { channelOf } from "../audio";
import { itemHasSound, mapItems, soundNodeId, walkItems } from "../compile/clip";
import type { AudioItem, TimelineSpec } from "../schema/timeline";
import {
  DEFAULT_BUS_FOR,
  DEFAULT_STRIP,
  dbToGain,
  defaultMix,
  type Bus,
  type ChannelKind,
  type Mix,
  type Strip,
} from "./types";
import { msToExactFrames, msToFrames } from "../timeline/time";
import { numberOr } from "../math";
import { findById } from "../lists";

export const resolveMix = (mix: Mix | undefined): Mix => mix ?? defaultMix();

export const stripOf = (mix: Mix, channel: string): Strip => mix.channels[channel] ?? DEFAULT_STRIP;

export function busOf(mix: Mix, channel: string, kind: ChannelKind): Bus {
  const strip = stripOf(mix, channel);
  return (
    findById(mix.buses, strip.bus) ?? findById(mix.buses, DEFAULT_BUS_FOR[kind]) ?? mix.buses[0]
  );
}

const kindOfAudio = (a: AudioItem): ChannelKind => (a.role === "music" ? "music" : "effect");

export function soloActive(mix: Mix): boolean {
  return Object.values(mix.channels).some((s) => s.solo) || mix.buses.some((b) => b.solo);
}

export function isAudible(mix: Mix, channel: string, kind: ChannelKind): boolean {
  const strip = stripOf(mix, channel);
  const bus = busOf(mix, channel, kind);
  if (strip.mute || bus.mute) return false;
  if (!soloActive(mix)) return true;
  return strip.solo || bus.solo;
}

function channelGain(mix: Mix, channel: string, kind: ChannelKind): number {
  if (!isAudible(mix, channel, kind)) return 0;
  const strip = stripOf(mix, channel);
  const bus = busOf(mix, channel, kind);
  return dbToGain(strip.trim + strip.fader) * dbToGain(bus.fader) * dbToGain(mix.master.fader);
}

export type ChannelInfo = {
  id: string;
  kind: ChannelKind;
  keepMusic: boolean;
  lengthFrames: number;
};

const playedFrames = (a: AudioItem): number =>
  a.loop ? Infinity : (a.lengthFrames ?? a.durationInFrames ?? Infinity);

const soundChannel = (a: AudioItem, prev: ChannelInfo | undefined): ChannelInfo => ({
  id: channelOf(a),
  kind: kindOfAudio(a),
  keepMusic: !!a.keepMusic && (prev?.keepMusic ?? true),
  lengthFrames: Math.max(prev?.lengthFrames ?? 0, playedFrames(a)),
});

const clipChannel = (id: string): ChannelInfo => ({
  id,
  kind: "clip",
  keepMusic: false,
  lengthFrames: Infinity,
});

export function channelsIn(spec: Pick<TimelineSpec, "tracks" | "audio">): ChannelInfo[] {
  const out = new Map<string, ChannelInfo>();
  for (const a of spec.audio) out.set(channelOf(a), soundChannel(a, out.get(channelOf(a))));
  walkItems(spec.tracks, (item) => {
    const id = soundNodeId(item);
    if (id) out.set(id, clipChannel(id));
  });
  return [...out.values()];
}

export function duckKeysFor(mix: Mix, bus: Bus, channels: ChannelInfo[], fps: number): string[] {
  if (!bus.duck.on) return [];
  const keys = new Set(bus.duck.keys);
  const shortest = msToExactFrames(bus.duck.ignoreShorterMs, fps);
  return channels
    .filter((c) => {
      const from = busOf(mix, c.id, c.kind).id;
      return !c.keepMusic && c.lengthFrames >= shortest && keys.has(from) && from !== bus.id;
    })
    .filter((c) => isAudible(mix, c.id, c.kind))
    .map((c) => c.id);
}

export function applyMixLevels(spec: TimelineSpec, mixIn: Mix | undefined): TimelineSpec {
  const mix = resolveMix(mixIn);
  const channels = channelsIn(spec);
  const duckFor = new Map<string, AudioItem["duck"]>();
  for (const bus of mix.buses) {
    const keys = duckKeysFor(mix, bus, channels, spec.fps);
    if (keys.length)
      duckFor.set(bus.id, {
        level: dbToGain(-bus.duck.amount),
        fadeFrames: msToFrames(bus.duck.attack, spec.fps),
        keys,
      });
  }
  const audio = spec.audio.map((a) => {
    const id = channelOf(a);
    const kind = kindOfAudio(a);
    return {
      ...a,
      gain: channelGain(mix, id, kind),
      duck: duckFor.get(busOf(mix, id, kind).id),
    };
  });
  const tracks = mapItems(spec.tracks, (item) => {
    const id = soundNodeId(item);
    if (!id) return item;
    const gain = channelGain(mix, id, "clip");
    const volume = numberOr(item.props.volume, 1);
    return {
      ...item,
      props: { ...item.props, volume: volume * gain, ...(gain === 0 ? { muted: true } : {}) },
    };
  });
  return { ...spec, audio, tracks };
}

export function withMasterStem(
  spec: TimelineSpec,
  master: { src: string; hash: string },
): TimelineSpec {
  const tracks = mapItems(spec.tracks, (item) =>
    itemHasSound(item) ? { ...item, props: { ...item.props, muted: true } } : item,
  );
  return { ...spec, tracks, audio: [], mix: master };
}
