import { z } from "zod";
import { patchRecordings } from "../compile/clip";
import type { AnnotationKind, AnnotationSpec, KeystrokePosition } from "../compile/recording";
import { LABELLED_TEXT_MAX, mutedIf, quote, row, toggleRow } from "./card";
import { defineNode, modifierBase, withClip, type NodeCategory, type NodeDef } from "./define";
import type { NodeIconName } from "./icon-name";
import { bool, choiceOf, int, more, ms, num, text } from "./params";

const base = modifierBase("Cursor");

const CHIP_POSITIONS: Record<KeystrokePosition, string> = {
  bottom: "Bottom",
  top: "Top",
};

const cursorStyleNode = defineNode({
  ...base,
  type: "cursor.style",
  title: "Cursor",
  description: "Shows or hides the drawn cursor and sets its size.",
  card: ({ params: p }) => [
    row("mouse-pointer", p.visible ? `Shown · ${p.size}×` : "Hidden", mutedIf(!p.visible)),
  ],
  params: z.object({
    visible: bool(true, "Show the cursor"),
    size: num(1, "Size", { min: 0.4, max: 3, step: 0.05, unit: "×" }),
  }),
  compile: ({ inputs, params }) =>
    withClip(inputs, (clip) =>
      patchRecordings(clip, (p) => ({ ...p, cursor: params.visible, cursorScale: params.size })),
    ),
});

const clickRippleNode = defineNode({
  ...base,
  type: "cursor.clickRipple",
  title: "Click ripple",
  description: "A ring that pulses on every click.",
  card: ({ params: p }) => [toggleRow("circle-dot", p.enabled)],
  params: z.object({ enabled: bool(true, "Ripple on every click") }),
  compile: ({ inputs, params }) =>
    withClip(inputs, (clip) => patchRecordings(clip, (p) => ({ ...p, ripple: params.enabled }))),
});

const keystrokeChipsNode = defineNode({
  ...base,
  type: "cursor.keystrokeChips",
  title: "Keystroke chips",
  description: "Shows keys and typed text as keycaps on screen.",
  card: ({ params: p }) => [
    row(
      "keyboard",
      [CHIP_POSITIONS[p.position], p.typed ? "typed text" : null, p.chords ? "shortcuts" : null]
        .filter((part) => part !== null)
        .join(" · "),
    ),
  ],
  params: z.object({
    position: choiceOf(CHIP_POSITIONS, "bottom", "Position"),
    typed: bool(true, "Show typed text"),
    chords: bool(true, "Show shortcuts"),
  }),
  compile: ({ inputs, params }) =>
    withClip(inputs, (clip) => patchRecordings(clip, (p) => ({ ...p, keystrokes: params }))),
});

type Annotation = Pick<NodeDef, "type" | "title" | "description"> & {
  category?: NodeCategory;
  icon: NodeIconName;
  kind: AnnotationKind;
  defaults: { text?: string; color: string };
};

export function annotationNode({ kind, defaults, icon, category = "Cursor", ...def }: Annotation) {
  return defineNode({
    ...modifierBase(category),
    ...def,
    card: ({ params: p }) => [
      row(
        icon,
        p.match.trim() ? `On actions named ${quote(p.match, LABELLED_TEXT_MAX)}` : "On every click",
      ),
      p.text.trim() && row("type", quote(p.text)),
    ],
    params: z.object({
      match: more(text("", "Show on actions named"), {
        help: "Leave empty to show on every click. Matches part of an action's name.",
      }),
      text: text(defaults.text ?? "", "Text"),
      color: more(text(defaults.color, "Colour"), { widget: "color" }),
      padding: int(10, "Space around", { min: 0, max: 80, unit: "px" }),
      leadMs: ms(250, "Appear early by", 5000),
      holdMs: ms(1400, "Stay for", 10_000),
    }),
    compile: ({ inputs, params }) =>
      withClip(inputs, (clip) =>
        patchRecordings(clip, (p) => ({
          ...p,
          annotations: [...p.annotations, { kind, ...params } satisfies AnnotationSpec],
        })),
      ),
  });
}

const spotlightNode = annotationNode({
  type: "cursor.spotlight",
  title: "Spotlight",
  kind: "spotlight",
  icon: "flashlight",
  description: "Darkens everything except the element being used.",
  defaults: { color: "rgba(0,0,0,0.55)" },
});
const highlightBoxNode = annotationNode({
  type: "cursor.highlightBox",
  title: "Highlight box",
  kind: "highlight",
  icon: "square-dashed",
  description: "Draws a box around the element being used.",
  defaults: { color: "#a78bfa" },
});

export const cursorNodes = [
  cursorStyleNode,
  clickRippleNode,
  keystrokeChipsNode,
  spotlightNode,
  highlightBoxNode,
];
