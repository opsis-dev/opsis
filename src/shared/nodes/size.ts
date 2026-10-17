import type { NodeDef } from "./define";

export const CARD_WIDTH = 256;
const HEADER = 50;
const PORT_ROW = 22;
const CARD_ROW = 20;
const THUMB = 156;
const PADDING = 16;
const MAX_CARD_ROWS = 4;
const UNKNOWN_HEIGHT = 120;
const NOTE_HEIGHT = 140;

function hasThumbnail(def: NodeDef): boolean {
  if (def.category === "Browser" || def.category === "Utility") return false;
  if ("clip" in def.outputs) return true;
  return !!def.executes && def.phase !== "post";
}

export function estimateCardHeight(def: NodeDef | undefined): number {
  if (!def) return UNKNOWN_HEIGHT;
  if (def.type === "core.note") return NOTE_HEIGHT;
  const ports = Math.max(Object.keys(def.inputs).length, Object.keys(def.outputs).length);
  const rows = def.card ? MAX_CARD_ROWS : 0;
  return HEADER + ports * PORT_ROW + rows * CARD_ROW + (hasThumbnail(def) ? THUMB : 0) + PADDING;
}
