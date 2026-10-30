export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const CANCELLED = "Cancelled";

export const NO_PROJECT = "No project is open.";
