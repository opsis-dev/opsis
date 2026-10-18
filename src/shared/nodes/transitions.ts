import { z } from "zod";
import { itemClip, sequenceClips, stackClips } from "../compile/clip";
import { ELEMENTS, getElementMeta } from "../elements";
import type { ElementMeta } from "../elements/types";
import type { TransitionRef } from "../schema/timeline";
import type { ClipValue } from "../schema/values";
import { cover, coverRow, frameSecs, row, sentence, squeezedTooMuch, type CardRow } from "./card";
import {
  clipInput,
  defineNode,
  fromElement,
  port,
  type CompileInputs,
  type CompileOutputs,
  type NodeDef,
} from "./define";
import { bool, frames, int, more, num } from "./params";

const PRESENTATIONS = ELEMENTS.filter((e) => e.kind === "presentation");
export const NO_TRANSITION = "none";
export const DEFAULT_TRANSITION = "core/fade";

export const transitionTitle = (key: string) =>
  key === NO_TRANSITION ? "Cut" : sentence(getElementMeta(key)?.title ?? key);
export const TRANSITION_KINDS: [string, ...string[]] = [
  NO_TRANSITION,
  ...PRESENTATIONS.map((p) => p.key),
];

const ab = {
  inputs: { a: port("clip", "First"), b: port("clip", "Then") },
  outputs: { clip: port("clip", "Clip") },
} as const;

const transitionNodeType = (key: string): string => `transition.${key.replace("/", ".")}`;

export const transitionRef = (
  kind: string,
  durationInFrames: number,
  keepWhole: boolean,
  params?: Record<string, unknown>,
): TransitionRef => ({ kind, durationInFrames, params, hold: keepWhole || undefined });

const joinClips = (
  inputs: CompileInputs,
  join: (a: ClipValue, b: ClipValue) => ClipValue,
): CompileOutputs => {
  const a = clipInput(inputs, "a");
  const b = clipInput(inputs, "b");
  return { clip: a && b ? join(a, b) : (a ?? b) };
};

function speedRow(length: number, designed: number, fps: number): CardRow | null {
  if (length <= 0 || !squeezedTooMuch(length, designed)) return null;
  return row("fast-forward", `${(designed / length).toFixed(1)}× faster than designed`, {
    tone: "warn",
    title: `This transition is made for about ${frameSecs(designed, fps)}. Squeezed this much it can look like a cut.`,
  });
}

function transitionNode(meta: ElementMeta): NodeDef {
  return defineNode({
    ...ab,
    ...fromElement(meta),
    type: transitionNodeType(meta.key),
    category: "Transitions",
    description: `Plays the first clip, then the second, joined with a ${sentence(meta.title).toLowerCase()} transition. Tip: cut where the picture is still.`,
    card: ({ params: p, settings, inputFrames }) => [
      row(
        "arrow-right-left",
        `${frameSecs(p.durationInFrames, settings.fps)}${p.keepWhole ? " · held" : ""}`,
      ),
      speedRow(p.durationInFrames, meta.defaultDurationInFrames, settings.fps),
      !p.keepWhole &&
        coverRow(cover(p.durationInFrames, [inputFrames?.a, inputFrames?.b]), settings.fps),
    ],
    icon: "blend",
    params: z.object({
      ...meta.schema.shape,
      durationInFrames: frames(meta.defaultDurationInFrames, "Length"),
      keepWhole: more(bool(false, "Keep both clips whole"), {
        help: "The transition plays over a still of the last and first frames, so neither clip loses screen time. The result gets longer by the transition.",
      }),
    }),
    compile: ({ nodeId, inputs, params }) => {
      const { durationInFrames, keepWhole, ...rest } = params;
      return joinClips(inputs, (a, b) =>
        sequenceClips(
          nodeId,
          [a, b],
          [
            undefined,
            transitionRef(meta.key, durationInFrames, keepWhole, { ...meta.fixedProps, ...rest }),
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
    seed: int(3, "Variation", { min: 0, max: 100 }),
    hueShift: num(0, "Colour shift", { min: 0, max: 360, unit: "°" }),
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
      const leakCenteredOnCut = a.durationInFrames - Math.round(params.durationInFrames / 2);
      return stackClips(nodeId, [cut, leak], [0, Math.max(0, leakCenteredOnCut)]);
    }),
});

export const transitionNodes: NodeDef[] = [
  ...PRESENTATIONS.map(transitionNode),
  lightLeakTransitionNode,
];
