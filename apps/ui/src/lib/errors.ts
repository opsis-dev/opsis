export const errorMessage = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

export const CANCELLED = "Cancelled";

export const NO_PROJECT = "No project is open.";
