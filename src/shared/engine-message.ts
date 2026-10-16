export const ENGINE_MISSING_MESSAGE =
  "The video engine isn't set up yet. Set it up in Settings → Engine.";

export const isEngineMissing = (error: string): boolean => error.includes(ENGINE_MISSING_MESSAGE);
