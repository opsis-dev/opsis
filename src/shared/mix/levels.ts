import { audioKind, channelOf } from "../audio";
import { itemHasSound, mapItems, soundNodeId, walkItems } from "../compile/clip";
import { findById } from "../lists";
import { numberOr } from "../math";
import type { AudioItem, TimelineSpec } from "../schema/timeline";
import { msToExactFrames, msToFrames } from "../timeline/time";
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

export const resolveMix = (mix: Mix | undefined): Mix => mix ?? defaultMix();

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

export const isAudible = (mix: Mix, channel: string, kind: ChannelKind): boolean =>
  audibility(mix)(channel, kind);

function channelGain(mix: Mix, audible: Audibility, channel: string, kind: ChannelKind): number {
  if (!audible(channel, kind)) return 0;
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

function soundChannel(a: AudioItem, prev: ChannelInfo | undefined): ChannelInfo {
  const everyEarlierKeepsMusic = prev?.keepMusic ?? true;
  return {
    id: channelOf(a),
    kind: audioKind(a),
    keepMusic: everyEarlierKeepsMusic && !!a.keepMusic,
    lengthFrames: Math.max(prev?.lengthFrames ?? 0, playedFrames(a)),
  };
}

const clipChannel = (id: string): ChannelInfo => ({
  id,
  kind: "clip",
  keepMusic: false,
  lengthFrames: Infinity,
});

export function channelsIn(spec: Pick<TimelineSpec, "tracks" | "audio">): ChannelInfo[] {
  const out = new Map<string, ChannelInfo>();
  for (const a of spec.audio) {
    const id = channelOf(a);
    out.set(id, soundChannel(a, out.get(id)));
  }
  walkItems(spec.tracks, (item) => {
    const id = soundNodeId(item);
    if (id) out.set(id, clipChannel(id));
  });
  return [...out.values()];
}

export function duckKeysFor(
  mix: Mix,
  bus: Bus,
  channels: ChannelInfo[],
  fps: number,
  audible: Audibility = audibility(mix),
): string[] {
  if (!bus.duck.on) return [];
  const keys = new Set(bus.duck.keys);
  const shortest = msToExactFrames(bus.duck.ignoreShorterMs, fps);
  return channels
    .filter((c) => {
      const from = busOf(mix, c.id, c.kind).id;
      return !c.keepMusic && c.lengthFrames >= shortest && keys.has(from) && from !== bus.id;
    })
    .filter((c) => audible(c.id, c.kind))
    .map((c) => c.id);
}

export function applyMixLevels(spec: TimelineSpec, mixIn: Mix | undefined): TimelineSpec {
  const mix = resolveMix(mixIn);
  const audible = audibility(mix);
  const channels = channelsIn(spec);
  const duckFor = new Map<string, AudioItem["duck"]>();
  for (const bus of mix.buses) {
    const keys = duckKeysFor(mix, bus, channels, spec.fps, audible);
    if (keys.length)
      duckFor.set(bus.id, {
        level: dbToGain(-bus.duck.amount),
        fadeFrames: msToFrames(bus.duck.attack, spec.fps),
        keys,
      });
  }
  const audio = spec.audio.map((a) => {
    const id = channelOf(a);
    const kind = audioKind(a);
    return {
      ...a,
      gain: channelGain(mix, audible, id, kind),
      duck: duckFor.get(busOf(mix, id, kind).id),
    };
  });
  const tracks = mapItems(spec.tracks, (item) => {
    const id = soundNodeId(item);
    if (!id) return item;
    const gain = channelGain(mix, audible, id, "clip");
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
