import type {
  AudioItem,
  EditHandles,
  TimelineItem,
  TimelineSpec,
  TimelineTrack,
  TransitionRef,
  VisualEnvelope,
} from "../schema/timeline";
import { hasKeys } from "../timeline/envelope";
import type { ClipValue, CompileValue, RecordingValue } from "../schema/values";
import type { WorkflowSettings } from "../schema/workflow";
import { clamp, round3 } from "../math";
import type { RecordingProps } from "./recording";

export type Crop = { x: number; y: number; w: number; h: number };

export type GroupProps = {
  tracks: TimelineTrack[];
  startFrom?: number;
  remap?: Array<[number, number]>;
  loopEvery?: number;
  crop?: Crop;
  wrapper?: { component: string; props: Record<string, unknown> };
  background?: string;
  design?: { width: number; height: number };
};

export const isClip = (v: CompileValue | undefined): v is ClipValue => v?.type === "clip";

export function emptyClip(durationInFrames = 1): ClipValue {
  return { type: "clip", durationInFrames, tracks: [], audio: [] };
}

export function itemClip(
  nodeId: string,
  component: string,
  props: Record<string, unknown>,
  durationInFrames: number,
  extra: {
    recording?: RecordingValue;
    audio?: AudioItem[];
    handles?: EditHandles;
    envelope?: VisualEnvelope;
  } = {},
): ClipValue {
  const duration = Math.max(1, Math.round(durationInFrames));
  return {
    type: "clip",
    durationInFrames: duration,
    tracks: [
      {
        id: `${nodeId}:t`,
        items: [
          {
            id: `${nodeId}:i`,
            from: 0,
            durationInFrames: duration,
            component,
            props,
            nodeId,
            ...(extra.handles ? { handles: extra.handles } : {}),
            ...(hasKeys(extra.envelope) ? { envelope: extra.envelope } : {}),
          },
        ],
      },
    ],
    audio: extra.audio ?? [],
    recording: extra.recording,
  };
}

export function wrapClip(
  clip: ClipValue,
  nodeId: string,
  props: Omit<GroupProps, "tracks"> = {},
  durationInFrames = clip.durationInFrames,
): ClipValue {
  const duration = Math.max(1, Math.round(durationInFrames));
  const group: TimelineItem = {
    id: `${nodeId}:g`,
    from: 0,
    durationInFrames: duration,
    component: "core/group",
    props: { ...props, tracks: clip.tracks } satisfies GroupProps,
    nodeId,
  };
  return {
    type: "clip",
    durationInFrames: duration,
    tracks: [{ id: `${nodeId}:t`, items: [group] }],
    audio: clip.audio,
    recording: clip.recording,
  };
}

export function withHandles(
  clip: ClipValue,
  handles: EditHandles,
  envelope?: VisualEnvelope,
): ClipValue {
  if (clip.tracks.length !== 1 || clip.tracks[0].items.length !== 1) return clip;
  const [track] = clip.tracks;
  const item = track.items[0];
  return {
    ...clip,
    tracks: [
      {
        ...track,
        items: [
          {
            ...item,
            handles: { ...item.handles, ...handles },
            ...(hasKeys(envelope) ? { envelope } : {}),
          },
        ],
      },
    ],
  };
}

export const shiftAudio = (audio: AudioItem[], offset: number, prefix: string): AudioItem[] =>
  audio.map((a) => ({ ...a, id: `${prefix}/${a.id}`, from: a.from + offset }));

export function sequenceClips(
  nodeId: string,
  clips: ClipValue[],
  transitions: Array<TransitionRef | undefined> = [],
  opts: { ports?: string[]; slotNode?: string } = {},
): ClipValue {
  if (clips.length === 0) return emptyClip();
  const items: TimelineItem[] = [];
  const audio: AudioItem[] = [];
  const remaps: Array<Array<[number, number]>> = [];
  let cursor = 0;
  clips.forEach((clip, i) => {
    let enter = i > 0 ? transitions[i] : undefined;
    if (enter) {
      const max = enter.hold
        ? enter.durationInFrames
        : Math.min(clips[i - 1].durationInFrames, clip.durationInFrames) - 1;
      const d = clamp(max, 0, enter.durationInFrames);
      enter = d > 0 ? { ...enter, durationInFrames: d } : undefined;
    }
    const d = enter?.durationInFrames ?? 0;
    const held = !!enter?.hold && d > 0;
    if (held && items[i - 1]) {
      const prev = items[i - 1];
      const last = clips[i - 1].durationInFrames - 1;
      const remap = remaps[i - 1].length ? remaps[i - 1] : [[0, 0] as [number, number]];
      remaps[i - 1] = [
        ...remap,
        [prev.durationInFrames - 1, last],
        [prev.durationInFrames - 1 + d, last],
      ];
      items[i - 1] = {
        ...prev,
        durationInFrames: prev.durationInFrames + d,
        props: { ...(prev.props as GroupProps), remap: remaps[i - 1] } satisfies GroupProps,
      };
    }
    const from = held ? cursor : Math.max(0, cursor - d);
    if (enter && items[i - 1]) items[i - 1] = { ...items[i - 1], exit: enter };
    remaps.push(
      held
        ? [
            [0, 0],
            [d, 0],
          ]
        : [],
    );
    const lead = held ? d : 0;
    items.push({
      id: `${nodeId}:s${i}`,
      from,
      durationInFrames: clip.durationInFrames + lead,
      component: "core/group",
      props: (held
        ? { tracks: clip.tracks, remap: remaps[i] }
        : { tracks: clip.tracks }) satisfies GroupProps,
      enter,
      ...(opts.ports?.[i]
        ? { slot: { nodeId: opts.slotNode ?? nodeId, port: opts.ports[i] } }
        : {}),
    });
    audio.push(...shiftAudio(clip.audio, from + lead, `${nodeId}:s${i}`));
    cursor = from + lead + clip.durationInFrames;
  });
  return {
    type: "clip",
    durationInFrames: Math.max(1, cursor),
    tracks: [{ id: `${nodeId}:seq`, items }],
    audio,
    recording: clips.find((c) => c.recording)?.recording,
  };
}

export function stackClips(
  nodeId: string,
  clips: ClipValue[],
  offsets: number[] = [],
  opts: { ports?: string[]; move?: (i: number) => EditHandles["move"] } = {},
): ClipValue {
  if (clips.length === 0) return emptyClip();
  const tracks: TimelineTrack[] = clips.map((clip, i) => {
    const move = opts.move?.(i);
    const port = opts.ports?.[i];
    return {
      id: `${nodeId}:l${i}`,
      items: [
        {
          id: `${nodeId}:l${i}:g`,
          from: offsets[i] ?? 0,
          durationInFrames: clip.durationInFrames,
          component: "core/group",
          props: { tracks: clip.tracks } satisfies GroupProps,
          ...(move ? { handles: { move } } : {}),
          ...(port ? { slot: { nodeId, port } } : {}),
        },
      ],
    };
  });
  const duration = Math.max(...clips.map((c, i) => c.durationInFrames + (offsets[i] ?? 0)));
  return {
    type: "clip",
    durationInFrames: duration,
    tracks,
    audio: clips.flatMap((c, i) => shiftAudio(c.audio, offsets[i] ?? 0, `${nodeId}:l${i}`)),
    recording: clips.find((c) => c.recording)?.recording,
  };
}

export function mapItems(
  tracks: TimelineTrack[],
  fn: (item: TimelineItem) => TimelineItem,
): TimelineTrack[] {
  return tracks.map((t) => ({
    ...t,
    items: t.items.map((item) => {
      const mapped = fn(item);
      if (mapped.component === "core/group" && Array.isArray((mapped.props as GroupProps).tracks)) {
        const props = mapped.props as GroupProps;
        return { ...mapped, props: { ...props, tracks: mapItems(props.tracks, fn) } };
      }
      return mapped;
    }),
  }));
}

const isRecordingItem = (item: TimelineItem) => item.component === "core/recording";

const recordingPropsOf = (item: TimelineItem) => item.props as Partial<RecordingProps>;

export function patchRecordings(
  clip: ClipValue,
  patch: (props: Partial<RecordingProps>) => Partial<RecordingProps>,
): ClipValue {
  return {
    ...clip,
    tracks: mapItems(clip.tracks, (item) =>
      isRecordingItem(item) ? { ...item, props: patch(recordingPropsOf(item)) } : item,
    ),
  };
}

export function recordingProps(clip: ClipValue): Array<Partial<RecordingProps>> {
  const collect = (tracks: TimelineTrack[]): Array<Partial<RecordingProps>> =>
    tracks.flatMap((track) =>
      track.items.flatMap((item) => [
        ...(isRecordingItem(item) ? [recordingPropsOf(item)] : []),
        ...(item.component === "core/group"
          ? collect((item.props as GroupProps).tracks ?? [])
          : []),
      ]),
    );
  return collect(clip.tracks);
}

export function hasRecording(clip: ClipValue): boolean {
  let found = false;
  mapItems(clip.tracks, (item) => {
    if (item.component === "core/recording") found = true;
    return item;
  });
  return found;
}

export function clipToSpec(clip: ClipValue, settings: WorkflowSettings): TimelineSpec {
  return {
    width: settings.width,
    height: settings.height,
    fps: settings.fps,
    durationInFrames: Math.max(1, clip.durationInFrames),
    background: settings.background ?? (settings.theme === "light" ? "#fafafa" : "#09090b"),
    tracks: clip.tracks,
    audio: clip.audio,
    fonts: settings.fonts,
  };
}

export type Span = { from: number; to: number };

export function nodeSpans(spec: TimelineSpec): Record<string, Span> {
  const spans: Record<string, Span> = {};
  const visit = (tracks: TimelineTrack[], offset: number, startFrom: number, limit: Span) => {
    for (const track of tracks) {
      for (const item of track.items) {
        const from = Math.max(limit.from, offset + item.from - startFrom);
        const to = Math.min(limit.to, offset + item.from - startFrom + item.durationInFrames);
        if (to <= from) continue;
        if (item.nodeId) {
          const s = spans[item.nodeId];
          spans[item.nodeId] = s
            ? { from: Math.min(s.from, from), to: Math.max(s.to, to) }
            : { from, to };
        }
        if (item.component === "core/group") {
          const p = item.props as GroupProps;
          visit(p.tracks ?? [], offset + item.from - startFrom, p.startFrom ?? 0, { from, to });
        }
      }
    }
  };
  visit(spec.tracks, 0, 0, { from: 0, to: spec.durationInFrames });
  return spans;
}

export function remapDuration(remap: Array<[number, number]>): number {
  return remap.length ? remap[remap.length - 1][0] : 0;
}

export function remapFrame(remap: Array<[number, number]>, frame: number): number {
  if (remap.length === 0) return frame;
  if (frame <= remap[0][0]) return remap[0][1];
  for (let i = 1; i < remap.length; i++) {
    const [o1, i1] = remap[i];
    const [o0, i0] = remap[i - 1];
    if (frame <= o1) return o1 === o0 ? i1 : i0 + ((frame - o0) / (o1 - o0)) * (i1 - i0);
  }
  const [oN, iN] = remap[remap.length - 1];
  return iN + (frame - oN);
}

export function remapFromSegments(
  innerDuration: number,
  segments: Array<[number, number, number]>,
): Array<[number, number]> {
  const out: Array<[number, number]> = [[0, 0]];
  let outPos = 0;
  let inPos = 0;
  const push = (inTo: number, rate: number) => {
    if (inTo <= inPos) return;
    outPos += (inTo - inPos) / rate;
    inPos = inTo;
    out.push([round3(outPos), inPos]);
  };
  for (const [a, b, rate] of segments) {
    push(Math.min(a, innerDuration), 1);
    push(Math.min(b, innerDuration), Math.max(0.05, rate));
  }
  push(innerDuration, 1);
  return out;
}
