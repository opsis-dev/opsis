export const msToFrames = (ms: number, fps: number, min = 1): number =>
  Math.max(min, Math.round((ms / 1000) * fps));

export const framesToMs = (frames: number, fps: number): number =>
  Math.round((frames * 1000) / fps);
