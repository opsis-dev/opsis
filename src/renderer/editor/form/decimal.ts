import { useState, type ChangeEvent } from "react";
import { round3 } from "@/shared/math";

export const trimNumber = (n: number): string => String(round3(n));

type UnitScale = { factor: number; unit?: string; time: boolean };

export function unitScale(unit: string | undefined, fps: number): UnitScale {
  if (unit === "ms") return { factor: 1000, unit: "s", time: true };
  if (unit === "frames") return { factor: fps, unit: "s", time: true };
  return { factor: 1, unit, time: false };
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
