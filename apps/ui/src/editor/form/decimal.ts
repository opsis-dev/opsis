import { round3 } from "@opsis/video/common/math";
import type { ChangeEvent } from "react";
import { parseNumberText } from "@/core/text-input";
import { useCommitDraft } from "@/lib/use-commit-draft";

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
  return scale.time ? Math.round(n * scale.factor) : n;
}

export function useDecimalInput(
  shown: string,
  applyOn: "edit" | "blur",
  apply: (text: string) => void,
) {
  const draft = useCommitDraft(shown, apply, applyOn);
  return {
    inputMode: "decimal" as const,
    value: draft.text,
    onChange: (e: ChangeEvent<HTMLInputElement>) => draft.change(e.target.value),
    onBlur: draft.commit,
  };
}
