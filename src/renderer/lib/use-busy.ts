import { useState } from "react";
import { attempt } from "./toast";

export function useBusy() {
  const [busy, setBusy] = useState<string | null>(null);
  const run = async (key: string, action: () => Promise<unknown>, failure?: string) => {
    setBusy(key);
    await attempt(action, failure);
    setBusy(null);
  };
  return { busy, run };
}
