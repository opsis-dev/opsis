import { clamp } from "../math";

export const msToFrames = (ms: number, fps: number, min = 1): number =>
  Math.max(min, Math.round((ms / 1000) * fps));

export const framesToMs = (frames: number, fps: number): number =>
  Math.round((frames * 1000) / Math.max(1, fps));

export const frameIn = (frame: number, total: number): number =>
  clamp(Math.round(frame), 0, Math.max(0, total - 1));

const pad2 = (n: number) => String(n).padStart(2, "0");

const clock = (totalSeconds: number) => ({
  h: Math.floor(totalSeconds / 3600),
  m: Math.floor((totalSeconds % 3600) / 60),
  s: totalSeconds % 60,
});

export function formatTime(frame: number, fps: number, mode: "seconds" | "timecode"): string {
  const f = Math.max(0, Math.round(frame));
  if (mode === "timecode") {
    const { h, m, s } = clock(Math.floor(f / fps));
    return [h, m, s, f % fps].map(pad2).join(":");
  }
  const tenths = Math.round((f / fps) * 10);
  const m = Math.floor(tenths / 600);
  const rest = tenths - m * 600;
  const text = rest % 10 === 0 ? pad2(rest / 10) : (rest / 10).toFixed(1).padStart(4, "0");
  return `${m}:${text}`;
}

export function chapterTime(ms: number): string {
  const { h, m, s } = clock(Math.floor(ms / 1000));
  return h ? `${h}:${pad2(m)}:${pad2(s)}` : `${m}:${pad2(s)}`;
}
