export type CropRect = { x: number; y: number; w: number; h: number };

export type CropSpec = {
  k?: number;
  cx?: number;
  cy?: number;
  dx?: number;
  dy?: number;
  rect?: CropRect;
  fill?: number;
  isolate?: boolean;
};

export type ResolvedCrop = {
  k: number;
  cx: number;
  cy: number;
  dx: number;
  dy: number;
};

export const COMPONENT_FILL = 0.85;

export const SHARPNESS_CEILING = 1.74;

export function resolveCrop(crop?: CropSpec): ResolvedCrop | undefined {
  if (!crop) return undefined;
  if (crop.rect) {
    const { x, y, w, h } = crop.rect;
    const k = (crop.fill ?? COMPONENT_FILL) / Math.max(w, h);
    return {
      k,
      cx: 0.5,
      cy: 0.5,
      dx: -(x + w / 2 - 0.5) * k,
      dy: -(y + h / 2 - 0.5) * k,
    };
  }
  return {
    k: crop.k ?? 1,
    cx: crop.cx ?? 0.5,
    cy: crop.cy ?? 0.5,
    dx: crop.dx ?? 0,
    dy: crop.dy ?? 0,
  };
}

export function cropClipPath(crop?: CropSpec): string | undefined {
  if (!crop?.rect || crop.isolate === false) return undefined;
  const { x, y, w, h } = crop.rect;
  const pc = (n: number) => `${(n * 100).toFixed(4)}%`;
  return `inset(${pc(y)} ${pc(1 - (x + w))} ${pc(1 - (y + h))} ${pc(x)})`;
}

export function cropUpscale(
  crop: CropSpec | undefined,
  sourceWidth: number,
  outputWidth: number,
): number {
  const resolved = resolveCrop(crop);
  if (!resolved || sourceWidth <= 0) return 1;
  return (outputWidth / sourceWidth) * resolved.k;
}
