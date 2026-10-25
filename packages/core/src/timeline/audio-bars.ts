import { audioKind, channelOf, soundLength } from "#core/audio";
import { videoOf } from "#core/compile/clip";
import { numberOr } from "#core/math";
import { CHANNEL_KINDS, type ChannelKind } from "#core/mix/types";
import type { AudioItem, TimelineSpec } from "#core/schema/timeline";
import type { AudioBar, VideoBar } from "#core/timeline/flatten";
import { msToExactFrames } from "#core/timeline/time";

const roleOrder = (role: ChannelKind): number => CHANNEL_KINDS.indexOf(role);

function soundBar(
  a: AudioItem,
  spec: Pick<TimelineSpec, "durationInFrames" | "fps">,
): AudioBar | null {
  const from = Math.max(0, a.from);
  const to = Math.min(spec.durationInFrames, a.from + soundLength(a, spec.durationInFrames));
  if (to <= from) return null;
  return {
    type: "audio",
    key: `audio:${a.id}`,
    id: a.id,
    ...(a.nodeId ? { nodeId: a.nodeId } : {}),
    channel: channelOf(a),
    src: a.src,
    role: audioKind(a),
    from,
    to,
    lane: 0,
    trimBefore: (a.trimBefore ?? 0) + Math.max(0, -a.from),
    loop: !!a.loop,
    volume: numberOr(a.volume, 1),
    ...(a.handles ? { handles: a.handles } : {}),
    item: a,
  };
}

function clipSoundBar(bar: VideoBar, fps: number): AudioBar | null {
  const media = videoOf(bar);
  const sound = bar.sound;
  if (!sound || !media?.src) return null;
  return {
    type: "audio",
    key: `clip:${bar.key}`,
    id: bar.key,
    nodeId: bar.nodeId,
    channel: bar.nodeId,
    src: media.src,
    role: "clip",
    from: sound.from,
    to: sound.to,
    lane: 0,
    trimBefore: Math.round(sound.contentStart + msToExactFrames(media.trimBeforeMs, fps)),
    loop: media.loop,
    volume: media.volume,
    bar,
  };
}

export function audioBars(
  spec: Pick<TimelineSpec, "audio" | "durationInFrames" | "fps">,
  bars: VideoBar[],
): AudioBar[] {
  return [
    ...spec.audio.map((a) => soundBar(a, spec)),
    ...bars.map((bar) => clipSoundBar(bar, spec.fps)),
  ]
    .filter((a) => a !== null)
    .toSorted((x, y) => roleOrder(x.role) - roleOrder(y.role) || x.from - y.from);
}
