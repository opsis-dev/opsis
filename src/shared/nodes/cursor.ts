import { z } from "zod";
import { patchRecordings } from "../compile/clip";
import {
  DEFAULT_CURSOR_LOOK,
  DEFAULT_KEYCAP,
  DEFAULT_RIPPLE,
  type AnnotationKind,
  type AnnotationSpec,
  type KeystrokePosition,
} from "../compile/recording";
import { fontProp } from "../fonts";
import { joinParts, LABELLED_TEXT_MAX, mutedIf, quote, row, toggleRow } from "./card";
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
  description: "Shows or hides the drawn cursor and sets its size and colours.",
  card: ({ params: p }) => [
    row("mouse-pointer", p.visible ? `Shown · ${p.size}×` : "Hidden", {
      ...mutedIf(!p.visible),
      ...(p.visible ? { swatches: [p.color, p.outline] } : {}),
    }),
  ],
  params: z.object({
    visible: bool(true, "Show the cursor"),
    size: num(1, "Size", { min: 0.4, max: 3, step: 0.05, unit: "×" }),
    color: more(text(DEFAULT_CURSOR_LOOK.fill, "Colour"), { widget: "color" }),
    outline: more(text(DEFAULT_CURSOR_LOOK.edge, "Outline colour"), { widget: "color" }),
  }),
  compile: ({ inputs, params }) =>
    withClip(inputs, (clip) =>
      patchRecordings(clip, (p) => ({
        ...p,
        cursor: params.visible,
        cursorScale: params.size,
        cursorLook: { fill: params.color, edge: params.outline },
      })),
    ),
});

const clickRippleNode = defineNode({
  ...base,
  type: "cursor.clickRipple",
  title: "Click ripple",
  description: "A ring that pulses on every click.",
  card: ({ params: p }) => [
    toggleRow("circle-dot", p.enabled),
    p.enabled && row("palette", `${p.size}×`, { swatches: [p.color] }),
  ],
  params: z.object({
    enabled: bool(true, "Ripple on every click"),
    color: more(text(DEFAULT_RIPPLE.color, "Colour"), { widget: "color" }),
    size: num(DEFAULT_RIPPLE.size, "Size", { min: 0.3, max: 4, step: 0.05, unit: "×" }),
    width: more(
      num(DEFAULT_RIPPLE.width, "Ring width", { min: 0.5, max: 12, step: 0.5, unit: "px" }),
      {
        advanced: true,
      },
    ),
  }),
  compile: ({ inputs, params: { enabled, color, size, width } }) =>
    withClip(inputs, (clip) =>
      patchRecordings(clip, (p) => ({ ...p, ripple: enabled ? { color, size, width } : null })),
    ),
});

const keystrokeChipsNode = defineNode({
  ...base,
  type: "cursor.keystrokeChips",
  title: "Keystroke chips",
  description: "Shows keys and typed text as keycaps on screen.",
  card: ({ params: p }) => [
    row(
      "keyboard",
      joinParts(
        CHIP_POSITIONS[p.position],
        p.typed ? "typed text" : null,
        p.chords ? "shortcuts" : null,
      ),
    ),
  ],
  params: z.object({
    position: choiceOf(CHIP_POSITIONS, "bottom", "Position"),
    typed: bool(true, "Show typed text"),
    chords: bool(true, "Show shortcuts"),
    keycap: more(text(DEFAULT_KEYCAP.keycap, "Keycap colour"), { widget: "color" }),
    ink: more(text(DEFAULT_KEYCAP.ink, "Text colour"), { widget: "color" }),
    size: num(1, "Size", { min: 0.4, max: 3, step: 0.05, unit: "×" }),
  }),
  compile: ({ inputs, params }) =>
    withClip(inputs, (clip) => patchRecordings(clip, (p) => ({ ...p, keystrokes: params }))),
});

type AnnotationText = { fontSize: number; textColor?: string };

type Annotation = Pick<NodeDef, "type" | "title" | "description"> & {
  category?: NodeCategory;
  icon: NodeIconName;
  kind: AnnotationKind;
  defaults: { text?: string; color: string };
  label?: AnnotationText;
};

function labelParams(label: AnnotationText | undefined) {
  if (!label) return {};
  return {
    ...(label.textColor
      ? { textColor: more(text(label.textColor, "Text colour"), { widget: "color" }) }
      : {}),
    fontSize: int(label.fontSize, "Font size", { min: 8, max: 160, unit: "px" }),
    font: more(text("", "Font"), {
      widget: "font",
      help: "Project font follows the fonts set for the whole project.",
    }),
  };
}

export function annotationNode({
  kind,
  defaults,
  icon,
  label,
  category = "Cursor",
  ...def
}: Annotation) {
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
      ...labelParams(label),
    }),
    compile: ({ inputs, params }) => {
      const { font, ...rest }: { font?: string } & Omit<AnnotationSpec, "kind"> = params;
      const spec: AnnotationSpec = {
        kind,
        ...rest,
        ...(fontProp(font) ? { font: fontProp(font) } : {}),
      };
      return withClip(inputs, (clip) =>
        patchRecordings(clip, (p) => ({ ...p, annotations: [...p.annotations, spec] })),
      );
    },
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
