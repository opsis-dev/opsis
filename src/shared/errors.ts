export const errorMessage = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

export const CANCELLED = "Cancelled";

export const NO_PROJECT = "No project is open.";

export class CancelledError extends Error {
  constructor() {
    super(CANCELLED);
    this.name = "CancelledError";
  }
}

export function throwIfCancelled(signal: AbortSignal | undefined): void {
  if (signal?.aborted) throw new CancelledError();
}

export function onAbort(signal: AbortSignal | undefined, fn: () => void): () => void {
  signal?.addEventListener("abort", fn, { once: true });
  return () => signal?.removeEventListener("abort", fn);
}

export const withoutCallName = (text: string): string =>
  text.split("\n")[0].replace(/^(locator|page|frame|elementHandle|mouse|keyboard)\.\w+:\s*/, "");
