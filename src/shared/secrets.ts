export const secretName = (raw: string | undefined): string => raw?.trim() || "default";

export const secretFileName = (raw: string | undefined): string =>
  secretName(raw).replace(/[^\w.-]+/g, "-");
