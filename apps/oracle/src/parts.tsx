import { useHeldLoad } from "@opsis/runtime/holds.ts";
import type { CSSProperties, ReactNode } from "react";

const GUTTER = 24;
const GAP = 16;
const COLUMNS = 4;
const CELL_WIDTH = 136;
const CELL_HEIGHT = 148;

export const INK = "#f8fafc";
export const PAPER = "#0f172a";
export const ACCENT = "#7c3aed";
export const WARM = "#f97316";
export const COOL = "#0ea5e9";
export const LEAF = "#22c55e";
export const ROSE = "#f43f5e";
export const SLATE = "#334155";

export const STAGE: CSSProperties = {
  position: "absolute",
  inset: 0,
  overflow: "hidden",
  background: PAPER,
  color: INK,
  fontFamily: "Inter",
  fontSize: 16,
};

export const SAMPLE = "Sphinx of black quartz";
export const PARAGRAPH = "Opsis turns takes into finished films, one frame at a time.";

export function cell(index: number, extra: CSSProperties = {}): CSSProperties {
  const column = index % COLUMNS;
  const row = Math.floor(index / COLUMNS);
  return {
    position: "absolute",
    left: GUTTER + column * (CELL_WIDTH + GAP),
    top: GUTTER + row * (CELL_HEIGHT + GAP),
    width: CELL_WIDTH,
    height: CELL_HEIGHT,
    ...extra,
  };
}

function decoded(url: string): Promise<void> {
  const picture = new Image();
  picture.src = url;
  return picture.decode();
}

function allDecoded(key: string): Promise<void[]> {
  return Promise.all(key.split("\n").map(decoded));
}

export function Preloaded({ urls, children }: { urls: string[]; children: ReactNode }) {
  const loaded = useHeldLoad("Loading pictures", urls.join("\n"), allDecoded);
  return loaded === null ? null : <>{children}</>;
}
