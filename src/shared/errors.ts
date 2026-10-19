export const errorMessage = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

export const errorCode = (error: unknown): string | undefined =>
  error instanceof Error && "code" in error && typeof error.code === "string"
    ? error.code
    : undefined;

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
