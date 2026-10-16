export const NONE = "__none";

export const orNone = (value: string): string => value || NONE;

export const fromNone = (value: unknown): string =>
  value == null || value === NONE ? "" : String(value);
