export const ENGINE_MISSING_MESSAGE =
  "The video engine isn't set up yet. Set it up in Settings → Engine.";

export type EngineStatus = {
  dir: string;
  headlessShell: string | null;
  ffmpeg: string | null;
  ffprobe: string | null;
  complete: boolean;
};

export type EngineInstallUpdate = {
  part: "ffmpeg" | "headless-shell";
  step: string;
  progress: number | null;
  message: string;
};

export type EngineInstallEvent = EngineInstallUpdate & { done?: boolean; error?: string };
