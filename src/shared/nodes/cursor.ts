import { z } from "zod";
import {
  DEFAULT_CURSOR_LOOK,
  DEFAULT_KEYCAP,
  DEFAULT_RIPPLE,
  HAND_DRAWN_LOOKS,
  type AnnotationKind,
  type AnnotationSpec,
  type CursorLook,
  type HighlightLook,
  type KeystrokePosition,
} from "../compile/recording";
import { styleStep, type StyleSteps } from "../compile/style";
import { fontProp } from "../fonts";
import { joinParts, LABELLED_TEXT_MAX, mutedIf, quote, row, toggleRow } from "./card";
import { defineNode, styleModifier, type NodeCategory, type NodeDef } from "./define";
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

const base = styleModifier("Cursor");

const CURSOR_SHAPES: Record<CursorLook["shape"], string> = {
  arrow: "Simple arrow",
  mac: "Mac",
};

const ARROW_ONLY = { showIf: { look: ["arrow"] } };

const CHIP_POSITIONS: Record<KeystrokePosition, string> = {
  bottom: "Bottom",
  top: "Top",
};

export const CURSOR_STYLE_TYPE = "cursor.style";

const cursorStyleNode = defineNode({
  ...base,
  type: CURSOR_STYLE_TYPE,
  title: "Cursor",
  description: "Shows or hides the drawn cursor and sets its look, size and colours.",
  card: ({ params: p }) => [
    row("mouse-pointer", p.visible ? `${CURSOR_SHAPES[p.look]} · ${p.size}×` : "Hidden", {
      ...mutedIf(!p.visible),
      ...(p.visible && p.look === "arrow"
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
    look: more(choiceOf(CURSOR_SHAPES, "arrow", "Look"), {
      help: "Mac turns into a pointing hand over links and buttons and a text cursor over fields, as the page asks.",
    }),
    size: num(1, "Size", { min: 0.4, max: 3, step: 0.05, unit: "×" }),
    color: more(colorSetting(DEFAULT_CURSOR_LOOK.fill, "Colour"), ARROW_ONLY),
    outline: more(colorSetting(DEFAULT_CURSOR_LOOK.edge, "Outline colour"), ARROW_ONLY),
  }),
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
});

type AnnotationText = { fontSize: number; textColor?: string };

const HIGHLIGHT_LOOK_LABELS: Record<HighlightLook, string> = {
  clean: "Clean",
  box: "Hand-drawn box",
  circle: "Circle",
  underline: "Underline",
  marker: "Marker",
};

const lookParams = (shown: boolean) => ({
  look: more(choiceOf(HIGHLIGHT_LOOK_LABELS, "clean", "Look"), {
    help: "The hand-drawn looks draw themselves in as the action starts.",
    hidden: !shown,
  }),
  roughness: more(num(1, "Sketchiness", { min: 0, max: 3, step: 0.1 }), {
    advanced: true,
    hidden: !shown,
    showIf: { look: [...HAND_DRAWN_LOOKS] },
  }),
});

type Annotation = Pick<NodeDef, "type" | "title" | "description"> & {
  category?: NodeCategory;
  icon: NodeIconName;
  kind: AnnotationKind;
  defaults: { text?: string; color: string };
  label?: AnnotationText;
  looks?: boolean;
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
  looks = false,
  category = "Cursor",
  ...def
}: Annotation) {
  const node = defineNode({
    ...styleModifier(category),
    ...def,
    card: ({ params: p }) => [
      row(
        icon,
        joinParts(
          p.match.trim()
            ? `On actions named ${quote(p.match, LABELLED_TEXT_MAX)}`
            : "On every action",
          p.look === "clean" ? null : HIGHLIGHT_LOOK_LABELS[p.look],
        ),
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
      ...lookParams(looks),
    }),
  });
  const step = styleStep(node, {
    restyle: (props, params) => {
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
      return { ...props, annotations: [...props.annotations, spec] };
    },
  });
  return { node, step };
}

const spotlight = annotationNode({
  type: "cursor.spotlight",
  title: "Spotlight",
  kind: "spotlight",
  icon: "flashlight",
  description: "Darkens everything except the element being used.",
  defaults: { color: "rgba(0,0,0,0.55)" },
});
const highlightBox = annotationNode({
  type: "cursor.highlightBox",
  title: "Highlight box",
  kind: "highlight",
  icon: "square-dashed",
  description: "Draws a box around the element being used, cleanly or by hand.",
  defaults: { color: "#a78bfa" },
  looks: true,
});

export const cursorNodes = [
  cursorStyleNode,
  clickRippleNode,
  keystrokeChipsNode,
  spotlight.node,
  highlightBox.node,
];

export const cursorStyles: StyleSteps = [
  styleStep(cursorStyleNode, {
    restyle: (props, p) => ({
      ...props,
      cursor: p.visible,
      cursorScale: p.size,
      cursorLook:
        p.look === "mac"
          ? { shape: "mac" }
          : {
              shape: "arrow",
              fill: colorOr(p.color, DEFAULT_CURSOR_LOOK.fill),
              edge: colorOr(p.outline, DEFAULT_CURSOR_LOOK.edge),
            },
    }),
  }),
  styleStep(clickRippleNode, {
    restyle: (props, { enabled, color, size, width }) => ({
      ...props,
      ripple: enabled ? { color: colorOr(color, DEFAULT_RIPPLE.color), size, width } : null,
    }),
  }),
  styleStep(keystrokeChipsNode, {
    restyle: (props, p) => ({
      ...props,
      keystrokes: {
        ...p,
        keycap: colorOr(p.keycap, DEFAULT_KEYCAP.keycap),
        ink: colorOr(p.ink, DEFAULT_KEYCAP.ink),
      },
    }),
  }),
  spotlight.step,
  highlightBox.step,
];
