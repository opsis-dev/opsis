import { useMemo, useState } from "react";
import { continueRender, delayRender } from "remotion";

export function holdRender(label: string): () => void {
  const handle = delayRender(label);
  let held = true;
  return () => {
    if (!held) return;
    held = false;
    continueRender(handle);
  };
}

export function useStableValue<T>(value: T): T {
  const key = useMemo(() => JSON.stringify(value), [value]);
  const [kept, setKept] = useState({ key, value });
  if (kept.key === key) return kept.value;
  setKept({ key, value });
  return value;
}
