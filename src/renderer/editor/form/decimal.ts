import { useState, type ChangeEvent } from "react";
import { parseNumberText } from "@/shared/format";
import { round3 } from "@/shared/math";

export const trimNumber = (n: number): string => String(round3(n));

type UnitScale = { factor: number; unit?: string; time: boolean };

export const MS_SCALE: UnitScale = { factor: 1000, unit: "s", time: true };

export const framesScale = (fps: number): UnitScale => ({ factor: fps, unit: "s", time: true });

export function unitScale(unit: string | undefined, fps: number): UnitScale {
  if (unit === "ms") return MS_SCALE;
  if (unit === "frames") return framesScale(fps);
  return { factor: 1, unit, time: false };
}

export const shownText = (stored: number, scale: UnitScale): string =>
  trimNumber(stored / scale.factor);

export function parseShown(text: string, scale: UnitScale): number | null {
  const n = parseNumberText(text);
  if (n === null) return null;
  return scale.time ? Math.round(n * scale.factor) : n * scale.factor;
}

export function useDecimalInput(
  shown: string,
  applyOn: "edit" | "blur",
  apply: (text: string) => void,
) {
  const [draft, setDraft] = useState<string | null>(null);
  return {
    inputMode: "decimal" as const,
    value: draft ?? shown,
    onChange: (e: ChangeEvent<HTMLInputElement>) => {
      setDraft(e.target.value);
      if (applyOn === "edit") apply(e.target.value);
    },
    onBlur: () => {
      if (applyOn === "blur" && draft !== null) apply(draft);
      setDraft(null);
    },
  };
}
