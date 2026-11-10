export function floatOffset(
  frame: number,
  fps: number,
  amplitude: number,
  startFrame: number,
  periodSeconds: number,
): number {
  const local = Math.max(0, frame - startFrame);
  const value = Math.sin((local / (fps * periodSeconds)) * Math.PI * 2) * amplitude;
  return value === 0 ? 0 : -value;
}
