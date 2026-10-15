export type ReelChip = {
  text: string;
  x: number;
  y: number;
  fromS: number;
  toS: number;
};

export function chipProblem(c: ReelChip, totalS: number): string | null {
  if (!c.text.trim()) return "a chip with no text";
  if (c.x < 0 || c.x > 1 || c.y < 0 || c.y > 1)
    return `chip "${c.text}" sits at (${c.x}, ${c.y}), outside the frame`;
  if (!(c.toS > c.fromS))
    return `chip "${c.text}" ends at ${c.toS}s, at or before its ${c.fromS}s start`;
  if (c.fromS >= totalS)
    return `chip "${c.text}" starts at ${c.fromS}s, past the ${totalS.toFixed(1)}s film`;
  return null;
}

export function chipsAt(chips: ReelChip[], t: number): ReelChip[] {
  return chips.filter((c) => t >= c.fromS && t < c.toS);
}
