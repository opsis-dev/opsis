import type { ActionKind, CursorFeel, CursorLook, HighlightLook } from "#video/generated/spec.ts";

export type ArrowCursorLook = Extract<CursorLook, { shape: "arrow" }>;

export const DEFAULT_CURSOR_LOOK: ArrowCursorLook = {
  shape: "arrow",
  fill: "#111418",
  edge: "#ffffff",
};

export const DEFAULT_CURSOR_FEEL: CursorFeel = {
  growWithZoom: 0.58,
  outlineWidth: 1.85,
  shadow: 0.34,
  pressSquash: 0.9,
  hideWhileTyping: true,
};

export const POINTER_KINDS = new Set<ActionKind>(["click", "type", "hover", "focus", "drag"]);

export type HandDrawnLook = Exclude<HighlightLook, "clean">;
