export function parseNumberText(text: string, { lenient = false } = {}): number | null {
  const normal = text.trim().replace(",", ".");
  if (normal === "") return null;
  const n = lenient ? Number.parseFloat(normal.replace(/[^\d.+-]/g, "")) : Number(normal);
  return Number.isFinite(n) ? n : null;
}

export function parseNumberList(text: string | undefined): number[] {
  return (text ?? "").split(",").map((part) => parseNumberText(part) ?? 0);
}

export function formatNumberList(parts: number[]): string {
  const trimmed = [...parts];
  while (trimmed.length > 1 && trimmed.at(-1) === 0) trimmed.pop();
  return trimmed.every((p) => p === 0) ? "" : trimmed.join(",");
}
