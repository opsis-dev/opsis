import { commands } from "@/generated/bindings";
import { reportError } from "@/lib/toast";

export function saveWithToast(): Promise<boolean> {
  return commands.saveProject().catch((e: unknown) => {
    reportError("Save failed", e);
    return false;
  });
}
