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
import {
  bool,
  choiceOf,
  colorOr,
  colorSetting,
  int,
  more,
  ms,
  num,
  requiredText,
  text,
} from "./params";

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
      ...(p.visible
        ? {
            swatches: [
              colorOr(p.color, DEFAULT_CURSOR_LOOK.fill),
              colorOr(p.outline, DEFAULT_CURSOR_LOOK.edge),
            ],
          }
        : {}),
    }),
  ],
  params: z.object({
    visible: bool(true, "Show the cursor"),
    size: num(1, "Size", { min: 0.4, max: 3, step: 0.05, unit: "×" }),
    color: colorSetting(DEFAULT_CURSOR_LOOK.fill, "Colour"),
    outline: colorSetting(DEFAULT_CURSOR_LOOK.edge, "Outline colour"),
  }),
  compile: ({ inputs, params }) =>
    withClip(inputs, (clip) =>
      patchRecordings(clip, (p) => ({
        ...p,
        cursor: params.visible,
        cursorScale: params.size,
        cursorLook: {
          fill: colorOr(params.color, DEFAULT_CURSOR_LOOK.fill),
          edge: colorOr(params.outline, DEFAULT_CURSOR_LOOK.edge),
        },
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
    p.enabled &&
      row("palette", `${p.size}×`, { swatches: [colorOr(p.color, DEFAULT_RIPPLE.color)] }),
  ],
  params: z.object({
    enabled: bool(true, "Ripple on every click"),
    color: colorSetting(DEFAULT_RIPPLE.color, "Colour"),
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
      patchRecordings(clip, (p) => ({
        ...p,
        ripple: enabled ? { color: colorOr(color, DEFAULT_RIPPLE.color), size, width } : null,
      })),
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
    keycap: colorSetting(DEFAULT_KEYCAP.keycap, "Keycap colour"),
    ink: colorSetting(DEFAULT_KEYCAP.ink, "Text colour"),
    size: num(1, "Size", { min: 0.4, max: 3, step: 0.05, unit: "×" }),
  }),
  compile: ({ inputs, params }) =>
    withClip(inputs, (clip) =>
      patchRecordings(clip, (p) => ({
        ...p,
        keystrokes: {
          ...params,
          keycap: colorOr(params.keycap, DEFAULT_KEYCAP.keycap),
          ink: colorOr(params.ink, DEFAULT_KEYCAP.ink),
        },
      })),
    ),
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
    ...(label.textColor ? { textColor: colorSetting(label.textColor, "Text colour") } : {}),
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
        p.match.trim()
          ? `On actions named ${quote(p.match, LABELLED_TEXT_MAX)}`
          : "On every action",
      ),
      p.text.trim() && row("type", quote(p.text)),
    ],
    params: z.object({
      match: more(text("", "Show on actions named"), {
        help: "Leave empty to show on every click, typing, hover and drag. Matches part of an action's name.",
      }),
      text: label
        ? requiredText("Text", "Add the text to show", defaults.text)
        : more(text("", "Text"), { hidden: true }),
      color: colorSetting(defaults.color, "Colour"),
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
        color: colorOr(rest.color, defaults.color),
        ...(label?.textColor && rest.textColor !== undefined
          ? { textColor: colorOr(rest.textColor, label.textColor) }
          : {}),
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
