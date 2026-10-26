import { reportError } from "@/lib/toast";

export const saveWithToast = (): Promise<boolean> =>
  window.opsis.saveProject().catch((e: unknown) => {
    reportError("Save failed", e);
    return false;
  });
