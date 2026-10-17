import { toast } from "sonner";
import { errorMessage } from "@/shared/errors";

export function reportError(title: string, error: unknown): void {
  toast.error(title, { description: errorMessage(error) });
}

function showError(error: unknown): void {
  toast.error(errorMessage(error));
}

export async function attempt(action: () => Promise<unknown>, failure?: string): Promise<void> {
  try {
    await action();
  } catch (error) {
    if (failure) reportError(failure, error);
    else showError(error);
  }
}
