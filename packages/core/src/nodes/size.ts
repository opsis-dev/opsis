import { MAX_CARD_ROWS } from "#core/nodes/card";
import type { NodeDef } from "#core/nodes/define";

export const CARD_WIDTH = 256;
export const CARD_GAP = 32;
export const COLUMN_WIDTH = 300;
export const ROW_HEIGHT = 170;
const HEADER = 50;
const PORT_ROW = 22;
const CARD_ROW = 20;
const PADDING = 16;
const UNKNOWN_HEIGHT = 120;

export function estimateCardHeight(def: NodeDef | undefined): number {
  if (!def) return UNKNOWN_HEIGHT;
  if (def.cardHeight) return def.cardHeight;
  const ports = Math.max(Object.keys(def.inputs).length, Object.keys(def.outputs).length);
  const rows = def.card ? MAX_CARD_ROWS : 0;
  return HEADER + ports * PORT_ROW + rows * CARD_ROW + PADDING;
}
