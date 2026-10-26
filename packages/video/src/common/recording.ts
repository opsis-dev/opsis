import type { ActionKind, CursorLook, HighlightLook } from "#video/generated/spec.ts";

export type ArrowCursorLook = Extract<CursorLook, { shape: "arrow" }>;

export const DEFAULT_CURSOR_LOOK: ArrowCursorLook = {
  shape: "arrow",
  fill: "#111418",
  edge: "#ffffff",
};

export const PREVIEW_FILE_WIDTH = 960;

export const POINTER_KINDS = new Set<ActionKind>(["click", "type", "hover", "focus", "drag"]);

export type HandDrawnLook = Exclude<HighlightLook, "clean">;
