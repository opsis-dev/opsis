export const errorMessage = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

export const CANCELLED = "Cancelled";

export class CancelledError extends Error {
  constructor() {
    super(CANCELLED);
    this.name = "CancelledError";
  }
}

export const isCancelled = (error: unknown): boolean =>
  error instanceof CancelledError || errorMessage(error) === CANCELLED;

export function throwIfCancelled(signal: AbortSignal | undefined): void {
  if (signal?.aborted) throw new CancelledError();
}

export const withoutCallName = (text: string): string =>
  text.split("\n")[0].replace(/^(locator|page|frame|elementHandle|mouse|keyboard)\.\w+:\s*/, "");
