import { z } from "zod";
import {
  emptyClip,
  isClip,
  msToFrames,
  sequenceClips,
  stackClips,
  withHandles,
  wrapClip,
} from "../compile/clip";
import type { AudioValue, ClipValue, CompileValue } from "../schema/values";
import { frameSecs, quote, row, secs } from "./card";
import { defineNode, opt, port } from "./define";
import { choice, frames, more, ms, text } from "./params";
import { TRANSITION_KINDS, transitionTitle } from "./transitions";

const base = { category: "Timeline", phase: "compile" } as const;
const IN_PORTS = ["in1", "in2", "in3", "in4", "in5", "in6"] as const;
const manyIn = Object.fromEntries(IN_PORTS.map((p, i) => [p, opt("clip", `Clip ${i + 1}`)]));
const layerIn = Object.fromEntries(
  IN_PORTS.map((p, i) => [
    p,
    opt(
      "clip",
      i === 0
        ? "Layer 1 (back)"
        : i === IN_PORTS.length - 1
          ? `Layer ${i + 1} (front)`
          : `Layer ${i + 1}`,
    ),
  ]),
);
const countIn = (connected: string[]) => IN_PORTS.filter((p) => connected.includes(p)).length;
const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;

const connectedClips = (
  inputs: Partial<Record<string, CompileValue>>,
): Array<{ port: string; clip: ClipValue }> =>
  IN_PORTS.flatMap((name) => {
    const clip = inputs[name];
    return isClip(clip) ? [{ port: name, clip }] : [];
  });

const single = {
  inputs: { clip: port("clip", "Clip") },
  outputs: { clip: port("clip", "Clip") },
} as const;

export const sequenceNode = defineNode({
  ...base,
  type: "timeline.sequence",
  title: "Sequence",
  icon: "list-video",
  description: "Plays clips one after another, with a transition between each.",
  inputs: manyIn,
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p, connected, clipFrames, settings }) => [
    row(
      "list-video",
      plural(countIn(connected), "clip"),
      countIn(connected) ? {} : { tone: "warn" },
    ),
    p.transition !== "none" &&
      row(
        "arrow-right-left",
        `${transitionTitle(p.transition)} · ${frameSecs(p.transitionDuration, settings.fps)}`,
      ),
    clipFrames && row("clock", frameSecs(clipFrames, settings.fps)),
  ],
  params: z.object({
    transition: z
      .enum(TRANSITION_KINDS)
      .default("core/fade")
      .meta({ description: "Transition", widget: "transition" }),
    transitionDuration: frames(15, "Transition length"),
  }),
  compile: ({ nodeId, inputs, params }) => {
    const connected = connectedClips(inputs);
    const clips = connected.map((c) => c.clip);
    const transitions = clips.map((_, i) =>
      i > 0 && params.transition !== "none"
        ? { kind: params.transition, durationInFrames: params.transitionDuration }
        : undefined,
    );
    return {
      clip: clips.length
        ? sequenceClips(nodeId, clips, transitions, { ports: connected.map((c) => c.port) })
        : emptyClip(30),
    };
  },
});

export const stackNode = defineNode({
  ...base,
  type: "timeline.stack",
  title: "Layers",
  icon: "layers",
  description: "Stacks clips on top of each other: Layer 1 at the back, the last at the front.",
  inputs: layerIn,
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p, connected, clipFrames, settings }) => [
    row("layers", plural(countIn(connected), "layer"), countIn(connected) ? {} : { tone: "warn" }),
    row(
      "clock",
      `Length follows ${p.duration === "first" ? "Layer 1" : p.duration === "last" ? "the front layer" : "the longest"}`,
    ),
    clipFrames && row("timer", frameSecs(clipFrames, settings.fps)),
  ],
  params: z.object({
    duration: choice(["longest", "first", "last"], "longest", "Length follows", {
      longest: "Longest",
      first: "Layer 1",
      last: "Front layer",
    }),
    offsetsMs: more(text("", "Start each layer at"), { widget: "offsets" }),
  }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const connected = connectedClips(inputs);
    const clips = connected.map((c) => c.clip);
    if (!clips.length) return { clip: emptyClip(30) };
    const offsets = params.offsetsMs
      .split(",")
      .map((s) => Number(s.trim()))
      .map((n) => (Number.isFinite(n) && n > 0 ? msToFrames(n, settings.fps) : 0));
    const stacked = stackClips(nodeId, clips, offsets, {
      ports: connected.map((c) => c.port),
      move: (index) => ({ nodeId, param: "offsetsMs", unit: "ms", index }),
    });
    const pick =
      params.duration === "first"
        ? clips[0]
        : params.duration === "last"
          ? clips[clips.length - 1]
          : null;
    return { clip: pick ? { ...stacked, durationInFrames: pick.durationInFrames } : stacked };
  },
});

export const trimNode = defineNode({
  ...base,
  ...single,
  type: "timeline.trim",
  title: "Trim",
  icon: "scissors",
  description: "Keeps only part of a clip.",
  card: ({ params: p }) => [
    row(
      "scissors",
      p.durationMs
        ? `${secs(p.startMs)} → ${secs(p.startMs + p.durationMs)}`
        : `From ${secs(p.startMs)} to the end`,
    ),
  ],
  params: z.object({
    startMs: ms(0, "Start"),
    durationMs: more(ms(0, "Keep"), { zeroLabel: "To the end" }),
  }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const clip = isClip(inputs.clip) ? inputs.clip : undefined;
    if (!clip) return {};
    const start = params.startMs ? msToFrames(params.startMs, settings.fps) : 0;
    const rest = Math.max(1, clip.durationInFrames - start);
    const duration = params.durationMs
      ? Math.min(rest, msToFrames(params.durationMs, settings.fps))
      : rest;
    return {
      clip: withHandles(wrapClip(clip, nodeId, { startFrom: start }, duration), {
        trimStart: {
          nodeId,
          param: "startMs",
          unit: "ms",
          linked: [{ param: "durationMs", sign: -1, ifNonZero: true }],
        },
        trimEnd: { nodeId, param: "durationMs", unit: "ms" },
      }),
    };
  },
});

export const loopNode = defineNode({
  ...base,
  ...single,
  type: "timeline.loop",
  title: "Repeat",
  icon: "repeat",
  description: "Plays a clip several times.",
  card: ({ params: p }) => [row("repeat", `${p.times}×`)],
  params: z.object({
    times: z.number().int().min(1).max(100).default(2).meta({ description: "Times", unit: "×" }),
  }),
  compile: ({ nodeId, inputs, params }) => {
    const clip = isClip(inputs.clip) ? inputs.clip : undefined;
    if (!clip) return {};
    return {
      clip: wrapClip(
        clip,
        nodeId,
        { loopEvery: clip.durationInFrames },
        clip.durationInFrames * params.times,
      ),
    };
  },
});

export const delayNode = defineNode({
  ...base,
  ...single,
  type: "timeline.delay",
  title: "Delay",
  icon: "timer",
  description: "Starts a clip later.",
  card: ({ params: p }) => [row("clock", `+${secs(p.ms)}`)],
  params: z.object({ ms: ms(500, "Delay") }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const clip = isClip(inputs.clip) ? inputs.clip : undefined;
    if (!clip) return {};
    return {
      clip: stackClips(nodeId, [clip], [params.ms ? msToFrames(params.ms, settings.fps) : 0], {
        move: () => ({ nodeId, param: "ms", unit: "ms" }),
      }),
    };
  },
});

export const speedNode = defineNode({
  ...base,
  ...single,
  type: "timeline.speed",
  title: "Speed",
  icon: "fast-forward",
  description: "Plays a clip faster or slower.",
  card: ({ params: p }) => [row("gauge", `${p.rate}×`)],
  params: z.object({
    rate: z
      .number()
      .min(0.1)
      .max(16)
      .default(1.5)
      .meta({ description: "Speed", step: 0.05, unit: "×" }),
  }),
  compile: ({ nodeId, inputs, params }) => {
    const clip = isClip(inputs.clip) ? inputs.clip : undefined;
    if (!clip) return {};
    const out = Math.max(1, Math.round(clip.durationInFrames / params.rate));
    return {
      clip: withHandles(
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
      ),
    };
  },
});

export const durationFitNode = defineNode({
  ...base,
  type: "timeline.durationFit",
  title: "Fit length",
  icon: "ruler",
  description:
    "Makes a clip a set length, or as long as a sound, by speeding up, holding or cutting.",
  inputs: { clip: port("clip", "Clip"), audio: opt("audio", "Fit to this sound") },
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p, connected }) => [
    row("ruler", connected.includes("audio") ? "Fits the sound" : secs(p.targetMs)),
    row(
      "settings-2",
      p.mode === "speed" ? "By speed" : p.mode === "hold" ? "By holding the end" : "By cutting",
    ),
  ],
  params: z.object({
    targetMs: more(ms(10_000, "Length"), { help: "Ignored when a sound is connected." }),
    mode: choice(["speed", "hold", "cut"], "speed", "How", {
      speed: "Change speed",
      hold: "Hold the end",
      cut: "Cut",
    }),
  }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const clip = isClip(inputs.clip) ? inputs.clip : undefined;
    if (!clip) return {};
    const audio = inputs.audio as AudioValue | undefined;
    const target = Math.max(
      1,
      audio?.durationInFrames ?? msToFrames(params.targetMs, settings.fps),
    );
    const d = clip.durationInFrames;
    const fixed = (out: ClipValue) =>
      audio?.durationInFrames
        ? out
        : withHandles(out, { trimEnd: { nodeId, param: "targetMs", unit: "ms" } });
    if (params.mode === "cut") return { clip: fixed(wrapClip(clip, nodeId, {}, target)) };
    if (params.mode === "hold") {
      return {
        clip: fixed(
          target <= d
            ? wrapClip(clip, nodeId, {}, target)
            : wrapClip(
                clip,
                nodeId,
                {
                  remap: [
                    [0, 0],
                    [d - 1, d - 1],
                    [target, d - 1],
                  ],
                },
                target,
              ),
        ),
      };
    }
    return {
      clip: fixed(
        wrapClip(
          clip,
          nodeId,
          {
            remap: [
              [0, 0],
              [target, d],
            ],
          },
          target,
        ),
      ),
    };
  },
});

export const branchNode = defineNode({
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
