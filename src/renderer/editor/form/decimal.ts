import { useState, type ChangeEvent } from "react";
import { round3 } from "@/shared/math";

export const trimNumber = (n: number): string => String(round3(n));

export type UnitScale = { factor: number; unit?: string; time: boolean };

export function unitScale(unit: string | undefined, fps: number): UnitScale {
  if (unit === "ms") return { factor: 1000, unit: "s", time: true };
  if (unit === "frames") return { factor: fps, unit: "s", time: true };
  return { factor: 1, unit, time: false };
}

export function useDecimalInput(
  shown: string,
  { onEdit, onCommit }: { onEdit?: (text: string) => void; onCommit?: (text: string) => void },
) {
  const [draft, setDraft] = useState<string | null>(null);
  return {
    inputMode: "decimal" as const,
    value: draft ?? shown,
    onChange: (e: ChangeEvent<HTMLInputElement>) => {
      setDraft(e.target.value);
      onEdit?.(e.target.value);
    },
    onBlur: () => {
      if (draft !== null) onCommit?.(draft);
      setDraft(null);
    },
  };
}
