import { z } from "zod";
import { patchRecordings } from "../compile/clip";
import type { AnnotationKind, AnnotationSpec } from "../compile/recording";
import { mutedIf, quote, row, toggleRow } from "./card";
import { clipModifier, defineNode, withClip, type NodeCategory } from "./define";
import { bool, choice, more, ms, text } from "./params";

const modifier = { ...clipModifier, phase: "compile" } as const;

const cursorStyleNode = defineNode({
  ...modifier,
  type: "cursor.style",
  title: "Cursor",
  category: "Cursor",
  icon: "mouse-pointer-2",
  description: "Shows or hides the drawn cursor and sets its size.",
  card: ({ params: p }) => [
    row("mouse-pointer", p.visible ? `Shown · ${p.size}×` : "Hidden", mutedIf(!p.visible)),
  ],
  params: z.object({
    visible: bool(true, "Show the cursor"),
    size: z
      .number()
      .min(0.4)
      .max(3)
      .default(1)
      .meta({ description: "Size", step: 0.05, unit: "×" }),
  }),
  compile: ({ inputs, params }) =>
    withClip(inputs, (clip) =>
      patchRecordings(clip, (p) => ({ ...p, cursor: params.visible, cursorScale: params.size })),
    ),
});

const clickRippleNode = defineNode({
  ...modifier,
  type: "cursor.clickRipple",
  title: "Click ripple",
  category: "Cursor",
  icon: "circle-dot",
  description: "A ring that pulses on every click.",
  card: ({ params: p }) => [toggleRow("circle-dot", p.enabled)],
  params: z.object({ enabled: bool(true, "Ripple on every click") }),
  compile: ({ inputs, params }) =>
    withClip(inputs, (clip) => patchRecordings(clip, (p) => ({ ...p, ripple: params.enabled }))),
});

const keystrokeChipsNode = defineNode({
  ...modifier,
  type: "cursor.keystrokeChips",
  title: "Keystroke chips",
  category: "Cursor",
  icon: "keyboard",
  description: "Shows keys and typed text as keycaps on screen.",
  card: ({ params: p }) => [
    row(
      "keyboard",
      [p.position === "top" ? "Top" : "Bottom", p.typed && "typed text", p.chords && "shortcuts"]
        .filter(Boolean)
        .join(" · "),
    ),
  ],
  params: z.object({
    position: choice(["bottom", "top"], "bottom", "Position", { bottom: "Bottom", top: "Top" }),
    typed: bool(true, "Show typed text"),
    chords: bool(true, "Show shortcuts"),
  }),
  compile: ({ inputs, params }) =>
    withClip(inputs, (clip) => patchRecordings(clip, (p) => ({ ...p, keystrokes: params }))),
});

function annotationNode(
  type: string,
  title: string,
  kind: AnnotationKind,
  category: NodeCategory,
  icon: string,
  description: string,
  defaults: { text?: string; color: string },
) {
  return defineNode({
    ...modifier,
    type,
    title,
    category,
    icon,
    description,
    card: ({ params: p }) => [
      row(icon, p.match.trim() ? `On actions named ${quote(p.match, 24)}` : "On every click"),
      p.text.trim() && row("type", quote(p.text)),
    ],
    params: z.object({
      match: more(text("", "Show on actions named"), {
        help: "Leave empty to show on every click. Matches part of an action's name.",
      }),
      text: text(defaults.text ?? "", "Text"),
      color: more(text(defaults.color, "Colour"), { widget: "color" }),
      padding: z
        .number()
        .int()
        .min(0)
        .max(80)
        .default(10)
        .meta({ description: "Space around", unit: "px" }),
      leadMs: ms(250, "Appear early by", 5000),
      holdMs: ms(1400, "Stay for", 10_000),
    }),
    compile: ({ inputs, params }) =>
      withClip(inputs, (clip) =>
        patchRecordings(clip, (p) => ({
          ...p,
          annotations: [...(p.annotations ?? []), { kind, ...params } satisfies AnnotationSpec],
        })),
      ),
  });
}

const spotlightNode = annotationNode(
  "cursor.spotlight",
  "Spotlight",
  "spotlight",
  "Cursor",
  "flashlight",
  "Darkens everything except the element being used.",
  { color: "rgba(0,0,0,0.55)" },
);
const highlightBoxNode = annotationNode(
  "cursor.highlightBox",
  "Highlight box",
  "highlight",
  "Cursor",
  "square-dashed",
  "Draws a box around the element being used.",
  { color: "#a78bfa" },
);
const tooltipNode = annotationNode(
  "text.tooltip",
  "Tooltip",
  "tooltip",
  "Text & overlays",
  "message-square",
  "A tooltip that stays attached to an element as the camera moves.",
  { text: "Click to continue", color: "#18181b" },
);
const calloutArrowNode = annotationNode(
  "text.calloutArrow",
  "Callout arrow",
  "callout",
  "Text & overlays",
  "move-up-right",
  "A hand-drawn arrow pointing at an element, with a label.",
  { text: "New!", color: "#f43f5e" },
);

export const cursorNodes = [
  cursorStyleNode,
  clickRippleNode,
  keystrokeChipsNode,
  spotlightNode,
  highlightBoxNode,
  tooltipNode,
  calloutArrowNode,
];
