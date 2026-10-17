import { z } from "zod";
import { itemClip, sequenceClips, stackClips } from "../compile/clip";
import { ELEMENTS, getElementMeta, type ElementMeta } from "../elements";
import type { ClipValue, CompileValue } from "../schema/values";
import { coverRow, frameSecs, row, sentence, TRANSITION_WARN_RATIO, type CardRow } from "./card";
import { clipInput, defineNode, port, type CompileOutputs, type NodeDef } from "./define";
import { bool, frames, more } from "./params";

const PRESENTATIONS = ELEMENTS.filter((e) => e.kind === "presentation");
export const TRANSITION_KINDS: [string, ...string[]] = ["none", ...PRESENTATIONS.map((p) => p.key)];

const ab = {
  inputs: { a: port("clip", "First"), b: port("clip", "Then") },
  outputs: { clip: port("clip", "Clip") },
} as const;

const keepWholeField = more(bool(false, "Keep both clips whole"), {
  help: "The transition plays over a still of the last and first frames, so neither clip loses screen time. The result gets longer by the transition.",
});

const joinClips = (
  inputs: Partial<Record<string, CompileValue>>,
  join: (a: ClipValue, b: ClipValue) => ClipValue,
): CompileOutputs => {
  const a = clipInput(inputs, "a");
  const b = clipInput(inputs, "b");
  return { clip: a && b ? join(a, b) : (a ?? b) };
};

function speedRow(length: number, designed: number, fps: number): CardRow | null {
  if (length <= 0 || length >= TRANSITION_WARN_RATIO * designed) return null;
  return row("fast-forward", `${(designed / length).toFixed(1)}× faster than designed`, {
    tone: "warn",
    title: `This transition is made for about ${frameSecs(designed, fps)}. Squeezed this much it can look like a cut.`,
  });
}

function transitionNode(meta: ElementMeta): NodeDef {
  return defineNode({
    ...ab,
    type: `transition.${meta.key.replace("/", ".")}`,
    title: sentence(meta.title),
    category: "Transitions",
    description: `Plays the first clip, then the second, joined with a ${sentence(meta.title).toLowerCase()} transition. Tip: cut where the picture is still.`,
    card: ({ params: p, settings, inputFrames }) => [
      row(
        "arrow-right-left",
        `${frameSecs(p.durationInFrames, settings.fps)}${p.keepWhole ? " · held" : ""}`,
      ),
      speedRow(p.durationInFrames, meta.defaultDurationInFrames, settings.fps),
      !p.keepWhole && coverRow(p.durationInFrames, Object.values(inputFrames ?? {}), settings.fps),
    ],
    icon: "blend",
    phase: "compile",
    element: meta.key,
    params: z.object({
      ...meta.schema.shape,
      durationInFrames: frames(meta.defaultDurationInFrames, "Length"),
      keepWhole: keepWholeField,
    }),
    compile: ({ nodeId, inputs, params }) => {
      const { durationInFrames, keepWhole, ...rest } = params;
      return joinClips(inputs, (a, b) =>
        sequenceClips(
          nodeId,
          [a, b],
          [
            undefined,
            {
              kind: meta.key,
              durationInFrames,
              params: { ...meta.fixedProps, ...rest },
              ...(keepWhole ? { hold: true } : {}),
            },
          ],
          { ports: ["a", "b"] },
        ),
      );
    },
  });
}

const lightLeakTransitionNode = defineNode({
  ...ab,
  type: "transition.lightLeak",
  title: "Light leak cut",
  category: "Transitions",
  description: "Cuts between two clips under a warm light flare.",
  icon: "sun",
  phase: "compile",
  card: ({ params, settings }) => [row("sun", frameSecs(params.durationInFrames, settings.fps))],
  params: z.object({
    durationInFrames: frames(30, "Length"),
    seed: z.number().int().min(0).max(100).default(3).meta({ description: "Variation" }),
    hueShift: z
      .number()
      .min(0)
      .max(360)
      .default(0)
      .meta({ description: "Colour shift", unit: "°" }),
  }),
  compile: ({ nodeId, inputs, params }) =>
    joinClips(inputs, (a, b) => {
      const cut = sequenceClips(`${nodeId}:cut`, [a, b], [], {
        ports: ["a", "b"],
        slotNode: nodeId,
      });
      const leak = itemClip(
        `${nodeId}:leak`,
        "core/light-leak",
        { seed: params.seed, hueShift: params.hueShift },
        params.durationInFrames,
      );
      return stackClips(
        nodeId,
        [cut, leak],
        [0, Math.max(0, a.durationInFrames - Math.round(params.durationInFrames / 2))],
      );
    }),
});

export const transitionNodes: NodeDef[] = [
  ...PRESENTATIONS.map(transitionNode),
  lightLeakTransitionNode,
];

export const transitionTitle = (key: string) =>
  key === "none" ? "Cut" : sentence(getElementMeta(key)?.title ?? key);
