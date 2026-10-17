import { safeName } from "./project-folder";

export const secretName = (raw: string | undefined): string => raw?.trim() || "default";

export const secretFileName = (raw: string | undefined): string =>
  safeName(secretName(raw), { fallback: "default", lower: false });
