import { z } from "zod";
import { isClip, itemClip, sequenceClips, stackClips } from "../compile/clip";
import { ELEMENTS, getElementMeta, type ElementMeta } from "../elements";
import { frameSecs, row, sentence } from "./card";
import { defineNode, port, type NodeDef } from "./define";
import { frames } from "./params";

export const PRESENTATIONS = ELEMENTS.filter((e) => e.kind === "presentation");
export const TRANSITION_KINDS = ["none", ...PRESENTATIONS.map((p) => p.key)] as [
  string,
  ...string[],
];

const ab = {
  inputs: { a: port("clip", "First"), b: port("clip", "Then") },
  outputs: { clip: port("clip", "Clip") },
} as const;

function transitionNode(meta: ElementMeta): NodeDef {
  return defineNode({
    ...ab,
    type: `transition.${meta.key.replace("/", ".")}`,
    title: sentence(meta.title),
    category: "Transitions",
    description: `Plays the first clip, then the second, joined with a ${sentence(meta.title).toLowerCase()} transition. Tip: cut where the picture is still.`,
    card: ({ params, settings }) => [
      row(
        "arrow-right-left",
        frameSecs(Number((params as Record<string, unknown>).durationInFrames ?? 0), settings.fps),
      ),
    ],
    icon: "blend",
    phase: "compile",
    element: meta.key,
    params: meta.schema.extend({
      durationInFrames: frames(meta.defaultDurationInFrames, "Length"),
    }),
    compile: ({ nodeId, inputs, params }) => {
      const a = isClip(inputs.a) ? inputs.a : undefined;
      const b = isClip(inputs.b) ? inputs.b : undefined;
      if (!a || !b) return { clip: a ?? b };
      const { durationInFrames, ...rest } = params as Record<string, unknown> & {
        durationInFrames: number;
      };
      return {
        clip: sequenceClips(
          nodeId,
          [a, b],
          [
            undefined,
            { kind: meta.key, durationInFrames, params: { ...meta.fixedProps, ...rest } },
          ],
          { ports: ["a", "b"] },
        ),
      };
    },
  });
}

export const lightLeakTransitionNode = defineNode({
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
  compile: ({ nodeId, inputs, params }) => {
    const a = isClip(inputs.a) ? inputs.a : undefined;
    const b = isClip(inputs.b) ? inputs.b : undefined;
    if (!a || !b) return { clip: a ?? b };
    const cut = sequenceClips(`${nodeId}:cut`, [a, b], [], { ports: ["a", "b"], slotNode: nodeId });
    const leak = itemClip(
      `${nodeId}:leak`,
      "core/light-leak",
      { seed: params.seed, hueShift: params.hueShift },
      params.durationInFrames,
    );
    return {
      clip: stackClips(
        nodeId,
        [cut, leak],
        [0, Math.max(0, a.durationInFrames - Math.round(params.durationInFrames / 2))],
      ),
    };
  },
});

export const transitionNodes: NodeDef[] = [
  ...PRESENTATIONS.map(transitionNode),
  lightLeakTransitionNode,
];

export const transitionTitle = (key: string) =>
  key === "none" ? "Cut" : sentence(getElementMeta(key)?.title ?? key);
