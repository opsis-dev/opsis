import { lookupKey } from "#core/hash";
import {
  audibility,
  busOf,
  channelsIn,
  duckKeysFor,
  resolveMix,
  stripOf,
  type Audibility,
  type ChannelInfo,
} from "#core/mix/levels";
import {
  stripSettings,
  type Bus,
  type ChannelKind,
  type Master,
  type Mix,
  type Strip,
  type StripSettings,
} from "#core/mix/types";
import type { AudioEnvelope, TimelineSpec } from "#core/schema/timeline";
import { flattenSpec, type FlatTimeline } from "#core/timeline/flatten";
import { atLeastOneFrame } from "#core/timeline/time";

export type SoundPlacement = {
  src: string;
  at: number;
  offset: number;
  length: number;
  loop: boolean;
  rate: number;
  volume: number;
  fadeIn: number;
  fadeOut: number;
  envelope?: AudioEnvelope;
};

export type ChannelPlan = {
  id: string;
  kind: ChannelKind;
  bus: string;
  strip: Strip;
  sounds: SoundPlacement[];
  audible: boolean;
};

export type BusPlan = {
  id: string;
  name: string;
  processing: Pick<Bus, "highpass" | "eq" | "comp">;
  fader: number;
  channels: string[];
  duck: { keys: string[]; amount: number; attack: number; release: number } | null;
};

export type MixPlan = {
  fps: number;
  durationInFrames: number;
  channels: ChannelPlan[];
  buses: BusPlan[];
  master: Master;
  key: string;
};

function soundPlacements(flat: FlatTimeline): Map<string, SoundPlacement[]> {
  const byChannel = new Map<string, SoundPlacement[]>();
  for (const a of flat.audio) {
    const list = byChannel.get(a.channel) ?? [];
    list.push({
      src: a.src,
      at: Math.round(a.from),
      offset: Math.max(0, Math.round(a.trimBefore)),
      length: atLeastOneFrame(a.to - a.from),
      loop: a.loop,
      rate: a.item?.rate ?? 1,
      volume: a.volume,
      fadeIn: a.item?.fadeInFrames ?? 0,
      fadeOut: a.item?.fadeOutFrames ?? 0,
      ...(a.item?.envelope ? { envelope: a.item.envelope } : {}),
    });
    byChannel.set(a.channel, list);
  }
  return byChannel;
}

function busPlans(
  mix: Mix,
  channels: ChannelPlan[],
  placed: ChannelInfo[],
  fps: number,
  audible: Audibility,
): BusPlan[] {
  return mix.buses.map((bus) => {
    const keys = duckKeysFor(mix, bus, placed, fps, audible);
    return {
      id: bus.id,
      name: bus.name,
      processing: { highpass: bus.highpass, eq: bus.eq, comp: bus.comp },
      fader: bus.fader,
      channels: channels.filter((c) => c.bus === bus.id && c.audible).map((c) => c.id),
      duck: keys.length
        ? { keys, amount: bus.duck.amount, attack: bus.duck.attack, release: bus.duck.release }
        : null,
    };
  });
}

export function buildMixPlan(
  spec: Pick<TimelineSpec, "tracks" | "audio" | "durationInFrames" | "fps">,
  mixIn: Mix | undefined,
  flat: FlatTimeline = flattenSpec(spec),
): MixPlan {
  const mix = resolveMix(mixIn);
  const audible = audibility(mix);
  const sounds = soundPlacements(flat);
  const placed = channelsIn(spec, flat).map((info) => ({
    ...info,
    sounds: sounds.get(info.id) ?? [],
  }));
  const channels: ChannelPlan[] = placed.map((c) => ({
    id: c.id,
    kind: c.kind,
    bus: busOf(mix, c.id, c.kind).id,
    strip: stripOf(mix, c.id),
    sounds: c.sounds,
    audible: audible(c.id, c.kind),
  }));
  const body = {
    fps: spec.fps,
    durationInFrames: spec.durationInFrames,
    channels,
    buses: busPlans(mix, channels, placed, spec.fps, audible),
    master: mix.master,
  };
  return { ...body, key: lookupKey(body) };
}

export const hasSound = (plan: MixPlan) => plan.channels.some((c) => c.sounds.length > 0);

type ChannelIdentity = Pick<MixPlan, "fps" | "durationInFrames"> &
  Pick<ChannelPlan, "sounds"> & { strip: StripSettings };

export function channelIdentity(plan: MixPlan, channel: ChannelPlan): ChannelIdentity {
  return {
    fps: plan.fps,
    durationInFrames: plan.durationInFrames,
    sounds: channel.sounds,
    strip: stripSettings(channel.strip),
  };
}
