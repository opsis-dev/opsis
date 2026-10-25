import { nonBlankString } from "#core/guards";
import { safeName } from "#core/safe-name";

export const secretName = (raw: unknown): string => nonBlankString(raw) ?? "default";

export const secretFileName = (raw: unknown): string =>
  safeName(secretName(raw), { fallback: "default", lower: false });
