export const msToFrames = (ms: number, fps: number, min = 1): number =>
  Math.max(min, Math.round((ms / 1000) * fps));

export const framesToMs = (frames: number, fps: number): number =>
  Math.round((frames * 1000) / fps);

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
  const secs = f / fps;
  const m = Math.floor(secs / 60);
  const s = secs - m * 60;
  const text = s.toFixed(s % 1 === 0 ? 0 : 1);
  return `${m}:${text.padStart(text.includes(".") ? 4 : 2, "0")}`;
}

export function chapterTime(ms: number): string {
  const { h, m, s } = clock(Math.floor(ms / 1000));
  return h ? `${h}:${pad2(m)}:${pad2(s)}` : `${m}:${pad2(s)}`;
}
