import { z } from "zod";
import {
  EMPTY_CLIP_FRAMES,
  emptyClip,
  sequenceClips,
  stackClips,
  withHandles,
  wrapClip,
  type GroupProps,
} from "../compile/clip";
import { asValue, type ClipValue, type CompileValue } from "../schema/values";
import type { WorkflowSettings } from "../schema/workflow";
import { countConnected, coverRow, frameSecs, quote, row, warnIf, type CardRow } from "./card";
import { clipInput, clipModifier, defineNode, opt, port, withClip } from "./define";
import { bool, choice, choiceOf, frames, int, MIN_SPEED, more, ms, rate, text } from "./params";
import { keepWholeField, TRANSITION_KINDS, transitionTitle } from "./transitions";
import { formatDuration, plural } from "../format";
import { msToFrames } from "../timeline/time";

const base = { category: "Timeline", phase: "compile" } as const;
export const IN_PORTS = ["in1", "in2", "in3", "in4", "in5", "in6"] as const;
const [, ...CUT_PORTS] = IN_PORTS;
type InPort = (typeof IN_PORTS)[number];

const clipLabel = (handle: InPort) => `Clip ${IN_PORTS.indexOf(handle) + 1}`;
const LAYER_ENDS: Partial<Record<InPort, string>> = { in1: " (back)", in6: " (front)" };
const layerLabel = (handle: InPort) =>
  `Layer ${IN_PORTS.indexOf(handle) + 1}${LAYER_ENDS[handle] ?? ""}`;

const manyIn = Object.fromEntries(IN_PORTS.map((p) => [p, opt("clip", clipLabel(p))]));
const layerIn = Object.fromEntries(IN_PORTS.map((p) => [p, opt("clip", layerLabel(p))]));

const connectedClips = (
  inputs: Partial<Record<string, CompileValue>>,
): Array<{ port: string; clip: ClipValue }> =>
  IN_PORTS.flatMap((name) => {
    const clip = clipInput(inputs, name);
    return clip ? [{ port: name, clip }] : [];
  });

const SLOW_PREVIEW_RATE = 4;

const CUT_KINDS = TRANSITION_KINDS.filter((k) => k !== "none");

const transitionFields = (length: string, keepWhole: { description: string; help: string }) => ({
  transition: z
    .enum(TRANSITION_KINDS)
    .default("core/fade")
    .meta({ description: "Transition", widget: "transition" }),
  transitionDuration: frames(15, length),
  keepWhole: more(keepWholeField(keepWhole.description, keepWhole.help), {
    showIf: { transition: CUT_KINDS },
  }),
});

export const cutSchema = z.object({
  before: choice(
    CUT_PORTS,
    "in2",
    "Before",
    Object.fromEntries(CUT_PORTS.map((handle) => [handle, clipLabel(handle)])),
  ),
  ...transitionFields("Length", {
    description: "Keep both clips whole",
    help: "The transition plays over a still of the two clips' last and first frames.",
  }),
});

type CutSettings = Omit<z.infer<typeof cutSchema>, "before">;

type Cut = CutSettings & { before: string; after: string; own: boolean };

function cutsFor(
  params: CutSettings & { cuts?: Array<CutSettings & { before: string }> },
  ports: readonly string[],
): Cut[] {
  return ports.slice(1).map((handle, i) => {
    const own = params.cuts?.findLast((c) => c.before === handle);
    const settings = own ?? params;
    return {
      transition: settings.transition,
      transitionDuration: settings.transitionDuration,
      keepWhole: settings.keepWhole,
      before: handle,
      after: ports[i],
      own: !!own,
    };
  });
}

function worstCover(
  cuts: readonly Cut[],
  inputFrames: Partial<Record<string, number>>,
  fps: number,
): CardRow | null {
  let worst: { cut: Cut; lengths: number[]; ratio: number } | null = null;
  for (const cut of cuts) {
    if (cut.transition === "none" || cut.keepWhole) continue;
    const lengths = [inputFrames[cut.after], inputFrames[cut.before]].filter(
      (f): f is number => f !== undefined && f > 0,
    );
    if (lengths.length === 0) continue;
    const ratio = cut.transitionDuration / Math.min(...lengths);
    if (!worst || ratio > worst.ratio) worst = { cut, lengths, ratio };
  }
  return worst ? coverRow(worst.cut.transitionDuration, worst.lengths, fps) : null;
}

const sequenceNode = defineNode({
  ...base,
  type: "timeline.sequence",
  title: "Sequence",
  icon: "list-video",
  description: "Plays clips one after another, with a transition between each.",
  inputs: manyIn,
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p, connected, clipFrames, settings, inputFrames }) => {
    const ports = IN_PORTS.filter((handle) => connected.includes(handle));
    const cuts = cutsFor(p, ports);
    const own = cuts.filter((c) => c.own).length;
    return [
      row("list-video", plural(ports.length, "clip"), warnIf(!ports.length)),
      p.transition !== "none" &&
        row(
          "arrow-right-left",
          `${transitionTitle(p.transition)} · ${frameSecs(p.transitionDuration, settings.fps)}${p.keepWhole ? " · held" : ""}`,
        ),
      own > 0 &&
        row(
          "arrow-right-left",
          `${plural(own, "cut")} ${own === 1 ? "uses its" : "use their"} own transition`,
        ),
      worstCover(cuts, inputFrames ?? {}, settings.fps),
      clipFrames && row("clock", frameSecs(clipFrames, settings.fps)),
    ];
  },
  params: z.object({
    ...transitionFields("Transition length", {
      description: "Keep every clip whole",
      help: "The transition plays over a still of the last and first frames, so no clip loses screen time. The sequence gets longer by each transition.",
    }),
    cuts: z.array(cutSchema).default([]).meta({
      description: "Different transitions",
      help: "Use another transition before some clips. The others use the transition above.",
    }),
  }),
  compile: ({ nodeId, inputs, params }) => {
    const connected = connectedClips(inputs);
    const clips = connected.map((c) => c.clip);
    const cuts = cutsFor(
      params,
      connected.map((c) => c.port),
    );
    const transitions = clips.map((_, i) => {
      const cut = i > 0 ? cuts[i - 1] : undefined;
      return cut && cut.transition !== "none"
        ? {
            kind: cut.transition,
            durationInFrames: cut.transitionDuration,
            ...(cut.keepWhole ? { hold: true } : {}),
          }
        : undefined;
    });
    return {
      clip: clips.length
        ? sequenceClips(nodeId, clips, transitions, { ports: connected.map((c) => c.port) })
        : emptyClip(EMPTY_CLIP_FRAMES),
    };
  },
});

type LengthSource = {
  label: string;
  follows: string;
  pick: (clips: ClipValue[]) => ClipValue | undefined;
};

const LENGTH_SOURCES = {
  longest: { label: "Longest", follows: "the longest", pick: () => undefined },
  first: { label: "Layer 1", follows: "Layer 1", pick: (clips) => clips[0] },
  last: { label: "Front layer", follows: "the front layer", pick: (clips) => clips.at(-1) },
} satisfies Record<string, LengthSource>;

const stackNode = defineNode({
  ...base,
  type: "timeline.stack",
  title: "Layers",
  icon: "layers",
  description: "Stacks clips on top of each other: Layer 1 at the back, the last at the front.",
  inputs: layerIn,
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p, connected, clipFrames, settings }) => {
    const layers = countConnected(IN_PORTS, connected);
    return [
      row("layers", plural(layers, "layer"), warnIf(!layers)),
      row("clock", `Length follows ${LENGTH_SOURCES[p.duration].follows}`),
      clipFrames && row("timer", frameSecs(clipFrames, settings.fps)),
    ];
  },
  params: z.object({
    duration: choiceOf(LENGTH_SOURCES, "longest", "Length follows"),
    offsetsMs: more(text("", "Start each layer at"), { widget: "offsets" }),
    keepLayout: more(bool(false, "Keep this layout in other shapes"), {
      help: "In 9:16, 1:1 and 4:5 the layers stay arranged as they are here, scaled to fit, so marks drawn over a picture stay on it.",
    }),
  }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const connected = connectedClips(inputs);
    const clips = connected.map((c) => c.clip);
    if (!clips.length) return { clip: emptyClip(EMPTY_CLIP_FRAMES) };
    const offsets = params.offsetsMs
      .split(",")
      .map((s) => Number(s.trim()))
      .map((n) => (Number.isFinite(n) && n > 0 ? msToFrames(n, settings.fps) : 0));
    const stacked = stackClips(nodeId, clips, offsets, {
      ports: connected.map((c) => c.port),
      move: (index) => ({ nodeId, param: "offsetsMs", unit: "ms", index }),
    });
    const pick = LENGTH_SOURCES[params.duration].pick(clips);
    const clip = pick ? { ...stacked, durationInFrames: pick.durationInFrames } : stacked;
    return { clip: params.keepLayout ? keepLayout(nodeId, clip, settings) : clip };
  },
});

function keepLayout(nodeId: string, clip: ClipValue, settings: WorkflowSettings): ClipValue {
  return {
    ...clip,
    tracks: [
      {
        id: `${nodeId}:design`,
        items: [
          {
            id: `${nodeId}:design:g`,
            from: 0,
            durationInFrames: clip.durationInFrames,
            component: "core/group",
            props: {
              tracks: clip.tracks,
              design: { width: settings.width, height: settings.height },
            } satisfies GroupProps,
          },
        ],
      },
    ],
  };
}

const trimNode = defineNode({
  ...base,
  ...clipModifier,
  type: "timeline.trim",
  title: "Trim",
  icon: "scissors",
  description: "Keeps only part of a clip.",
  card: ({ params: p }) => [
    row(
      "scissors",
      p.durationMs
        ? `${formatDuration(p.startMs)} → ${formatDuration(p.startMs + p.durationMs)}`
        : `From ${formatDuration(p.startMs)} to the end`,
    ),
  ],
  params: z.object({
    startMs: ms(0, "Start"),
    durationMs: more(ms(0, "Keep"), { zeroLabel: "To the end" }),
  }),
  compile: ({ nodeId, inputs, params, settings }) =>
    withClip(inputs, (clip) => {
      const start = params.startMs ? msToFrames(params.startMs, settings.fps) : 0;
      const rest = Math.max(1, clip.durationInFrames - start);
      const duration = params.durationMs
        ? Math.min(rest, msToFrames(params.durationMs, settings.fps))
        : rest;
      return withHandles(wrapClip(clip, nodeId, { startFrom: start }, duration), {
        trimStart: {
          nodeId,
          param: "startMs",
          unit: "ms",
          linked: [{ param: "durationMs", sign: -1, ifNonZero: true }],
        },
        trimEnd: { nodeId, param: "durationMs", unit: "ms" },
      });
    }),
});

const loopNode = defineNode({
  ...base,
  ...clipModifier,
  type: "timeline.loop",
  title: "Repeat",
  icon: "repeat",
  description: "Plays a clip several times.",
  card: ({ params: p }) => [row("repeat", `${p.times}×`)],
  params: z.object({
    times: int(2, "Times", { min: 1, max: 100, unit: "×" }),
  }),
  compile: ({ nodeId, inputs, params }) =>
    withClip(inputs, (clip) =>
      wrapClip(
        clip,
        nodeId,
        { loopEvery: clip.durationInFrames },
        clip.durationInFrames * params.times,
      ),
    ),
});

const delayNode = defineNode({
  ...base,
  ...clipModifier,
  type: "timeline.delay",
  title: "Delay",
  icon: "timer",
  description: "Starts a clip later.",
  card: ({ params: p }) => [row("clock", `+${formatDuration(p.ms)}`)],
  params: z.object({ ms: ms(500, "Delay") }),
  compile: ({ nodeId, inputs, params, settings }) =>
    withClip(inputs, (clip) =>
      stackClips(nodeId, [clip], [params.ms ? msToFrames(params.ms, settings.fps) : 0], {
        move: () => ({ nodeId, param: "ms", unit: "ms" }),
      }),
    ),
});

const speedNode = defineNode({
  ...base,
  ...clipModifier,
  type: "timeline.speed",
  title: "Speed",
  icon: "fast-forward",
  description: "Plays a clip faster or slower.",
  card: ({ params: p }) => [
    row("gauge", `${p.rate}×`),
    p.rate >= SLOW_PREVIEW_RATE &&
      row("info", "Very fast: the Preview plays slowly. The export is smooth.", {
        tone: "muted",
      }),
  ],
  params: z.object({
    rate: rate(1.5, "Speed", { min: MIN_SPEED }),
  }),
  compile: ({ nodeId, inputs, params }) =>
    withClip(inputs, (clip) => {
      const out = Math.max(1, Math.round(clip.durationInFrames / params.rate));
      return withHandles(
        wrapClip(
          clip,
          nodeId,
          {
            remap: [
              [0, 0],
              [out, clip.durationInFrames],
            ],
          },
          out,
        ),
        { speed: { nodeId, param: "rate", unit: "rate" } },
      );
    }),
});

type FitMode = {
  label: string;
  row: string;
  remap: (length: number, target: number) => GroupProps["remap"];
  warning: (input: number, output: number, fps: number) => CardRow | null;
};

const FIT_MODES = {
  speed: {
    label: "Change speed",
    row: "By speed",
    remap: (length, target) => [
      [0, 0],
      [target, length],
    ],
    warning: (input, output) => {
      const speed = input / output;
      return Math.abs(speed - 1) > 0.02
        ? row("fast-forward", `Plays at ${speed.toFixed(2)}×`, warnIf(speed > 1.5 || speed < 0.67))
        : null;
    },
  },
  hold: {
    label: "Hold the end",
    row: "By holding the end",
    remap: (length, target) =>
      target <= length
        ? undefined
        : [
            [0, 0],
            [length - 1, length - 1],
            [target, length - 1],
          ],
    warning: () => null,
  },
  cut: {
    label: "Cut",
    row: "By cutting",
    remap: () => undefined,
    warning: (input, output, fps) =>
      input > output
        ? row(
            "scissors",
            `Cuts ${frameSecs(input - output, fps)} off the end`,
            warnIf(input - output > fps / 2),
          )
        : null,
  },
} satisfies Record<string, FitMode>;

const fitRow = (
  mode: keyof typeof FIT_MODES,
  input: number | undefined,
  output: number | undefined,
  fps: number,
): CardRow | null => (input && output ? FIT_MODES[mode].warning(input, output, fps) : null);

const durationFitNode = defineNode({
  ...base,
  type: "timeline.durationFit",
  title: "Fit length",
  icon: "ruler",
  description:
    "Makes a clip a set length, or as long as a sound, by speeding up, holding or cutting.",
  inputs: { clip: port("clip", "Clip"), audio: opt("audio", "Fit to this sound") },
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p, connected, clipFrames, inputFrames, settings }) => [
    row("ruler", connected.includes("audio") ? "Fits the sound" : formatDuration(p.targetMs)),
    row("settings-2", FIT_MODES[p.mode].row),
    fitRow(p.mode, inputFrames?.clip, clipFrames, settings.fps),
  ],
  params: z.object({
    targetMs: more(ms(10_000, "Length"), { help: "Ignored when a sound is connected." }),
    mode: choiceOf(FIT_MODES, "speed", "How"),
  }),
  compile: ({ nodeId, inputs, params, settings }) =>
    withClip(inputs, (clip) => {
      const soundFrames = asValue(inputs.audio, "audio")?.durationInFrames;
      const target = Math.max(1, soundFrames ?? msToFrames(params.targetMs, settings.fps));
      const remap = FIT_MODES[params.mode].remap(clip.durationInFrames, target);
      const fitted = wrapClip(clip, nodeId, remap ? { remap } : {}, target);
      return soundFrames
        ? fitted
        : withHandles(fitted, { trimEnd: { nodeId, param: "targetMs", unit: "ms" } });
    }),
});

const branchNode = defineNode({
  ...base,
  type: "timeline.branch",
  title: "Choose by variable",
  icon: "split",
  description:
    "Uses the first clip when a variable has a value, otherwise the second. Handy for A/B versions.",
  inputs: { a: opt("clip", "First"), b: opt("clip", "Otherwise") },
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p }) => [
    p.variable.trim()
      ? row("git-branch", `${p.variable} = ${quote(p.equals, 20)}`)
      : row("git-branch", "Choose a variable", { tone: "warn" }),
  ],
  params: z.object({
    variable: more(text("Version", "Variable"), { widget: "variable" }),
    equals: text("A", "Equals"),
  }),
  compile: ({ inputs, params, vars }) => ({
    clip: (vars[params.variable] ?? "") === params.equals ? inputs.a : inputs.b,
  }),
});

export const timelineNodes = [
  sequenceNode,
  stackNode,
  trimNode,
  loopNode,
  delayNode,
  speedNode,
  durationFitNode,
  branchNode,
];
