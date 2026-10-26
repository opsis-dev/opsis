export type Span = readonly [number, number];

function spanGain(frame: number, [start, end]: Span, level: number, fadeFrames: number): number {
  if (frame >= start && frame < end) return level;
  const away = frame < start ? start - frame : frame - end;
  if (fadeFrames <= 0 || away >= fadeFrames) return 1;
  return level + (away / fadeFrames) * (1 - level);
}

export function duckGain(
  frame: number,
  spans: readonly Span[],
  level: number,
  fadeFrames: number,
): number {
  let gain = 1;
  for (const span of spans) gain = Math.min(gain, spanGain(frame, span, level, fadeFrames));
  return gain;
}
