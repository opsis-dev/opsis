export function shutterOffsets(shutterAngle: number, samples: number): number[] {
  const open = Math.min(Math.max(shutterAngle, 0), 360) / 360;
  if (open === 0 || samples <= 1) return [0];
  return Array.from({ length: samples }, (_, i) => (open * i) / samples);
}

export function sampleOpacity(index: number): number {
  return 1 / (index + 1);
}
