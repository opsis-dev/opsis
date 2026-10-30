import { useState } from "react";
import { attempt } from "./toast";

export function useBusy() {
  const [busy, setBusy] = useState<string | null>(null);
  async function run(key: string, action: () => Promise<unknown>, failure?: string) {
    setBusy(key);
    await attempt(action, failure);
    setBusy((current) => (current === key ? null : current));
  }
  return { busy, isBusy: busy !== null, run };
}
