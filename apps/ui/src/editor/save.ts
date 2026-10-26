import { commands } from "@/generated/bindings";
import { reportError } from "@/lib/toast";

export const saveWithToast = (): Promise<boolean> =>
  commands.saveProject().catch((e: unknown) => {
    reportError("Save failed", e);
    return false;
  });
