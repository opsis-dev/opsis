export type Canvas = { width: number; height: number };

export type ShotMeta = {
  name: string;
  region: Canvas;
  scale: number;
  viewport: Canvas;
  via?: string;
  backdrop?: string;
  ground?: string;
  split?: number;
  gutter?: number;
  look?: StillLook;
  fit?: number;
  anchor?: Anchor;
  inset?: number;
  zoom?: Zoom;
  shadow?: boolean;
  upscale?: number;
  body?: Box;
};

export type Box = { x: number; y: number; width: number; height: number };

export const STILL_LOOKS = ["window", "cutout", "bleed"] as const;
export type StillLook = (typeof STILL_LOOKS)[number];
export const DEFAULT_LOOK: StillLook = "window";

export const isLook = (v: unknown): v is StillLook =>
  typeof v === "string" && (STILL_LOOKS as readonly string[]).includes(v);

export function resolveLook(raw?: string | null): StillLook {
  if (raw == null || raw === "") return DEFAULT_LOOK;
  if (isLook(raw)) return raw;
  throw new Error(`unknown look "${raw}" — expected one of ${STILL_LOOKS.join(", ")}`);
}

export const ANCHORS = [
  "top-left",
  "top",
  "top-right",
  "left",
  "center",
  "right",
  "bottom-left",
  "bottom",
  "bottom-right",
] as const;
export type Anchor = (typeof ANCHORS)[number];

export const isAnchor = (v: unknown): v is Anchor =>
  typeof v === "string" && (ANCHORS as readonly string[]).includes(v);

export function resolveAnchor(raw?: string | null): Anchor {
  if (raw == null || raw === "") return "center";
  if (isAnchor(raw)) return raw;
  throw new Error(`unknown anchor "${raw}" — expected one of ${ANCHORS.join(", ")}`);
}

export const CUTOUT_FIT = 0.8;
export const MAX_CUTOUT_FIT = 2;
export const MAX_UPSCALE = 1.5;
export const BLEED_ZOOM = 1.3;
export type Zoom = number | "fill";

export const isZoom = (v: unknown): v is Zoom => v === "fill" || (typeof v === "number" && v >= 1);

export const fillZoom = (base: Canvas, canvas: Canvas): number =>
  Math.max(canvas.width / base.width, canvas.height / base.height);
export const BLEED_INSET = 0.06;

export const SURFACE_BUDGET_PX = 1920 * 1080 * 4 * 4;
export const MAX_SCALE = 8;

export function viewportFor(scale: number, like: Canvas): Canvas {
  const area = SURFACE_BUDGET_PX / (scale * scale);
  const aspect = like.height / like.width;
  const width = Math.floor(Math.sqrt(area / aspect) / 10) * 10;
  return { width, height: Math.round(width * aspect) };
}

export function maxScaleFor(viewport: Canvas): number {
  const byArea = Math.sqrt(SURFACE_BUDGET_PX / (viewport.width * viewport.height));
  return Math.max(1, Math.min(MAX_SCALE, Math.floor(byArea * 10) / 10));
}

export const shotPixels = (meta: ShotMeta): Canvas => ({
  width: Math.round(meta.region.width * meta.scale),
  height: Math.round(meta.region.height * meta.scale),
});

export const shotAspect = (meta: ShotMeta): number => meta.region.height / meta.region.width;

export const STILL_SHORT_EDGE = 2160;

export type StillPresetId = "wide" | "og" | "square" | "portrait" | "story";

export const STILL_PRESETS: Record<StillPresetId, Canvas> = {
  wide: { width: 3840, height: 2160 },
  og: { width: 2400, height: 1260 },
  square: { width: 2160, height: 2160 },
  portrait: { width: 2160, height: 2700 },
  story: { width: 2160, height: 3840 },
};

export const DEFAULT_PRESET: StillPresetId = "wide";

export const maxWindowPx = (fit: number): number =>
  Math.round(
    Math.max(
      ...STILL_PRESET_IDS.map((id) => Math.max(STILL_PRESETS[id].width, STILL_PRESETS[id].height)),
    ) * fit,
  );

export const STILL_PRESET_IDS = Object.keys(STILL_PRESETS) as StillPresetId[];

export const isPresetId = (v: unknown): v is StillPresetId =>
  typeof v === "string" && v in STILL_PRESETS;

export function resolvePreset(raw?: string | null): StillPresetId {
  if (raw == null || raw === "") return DEFAULT_PRESET;
  if (isPresetId(raw)) return raw;
  throw new Error(`unknown preset "${raw}" — expected one of ${STILL_PRESET_IDS.join(", ")}`);
}

export function windowBox(regionAspect: number, canvas: Canvas, fit: number): Canvas {
  if (!(regionAspect > 0)) throw new Error("regionAspect must be > 0");
  if (!(fit > 0)) throw new Error("fit must be > 0");
  let width = canvas.width * fit;
  let height = width * regionAspect;
  const maxHeight = canvas.height * fit;
  if (height > maxHeight) {
    height = maxHeight;
    width = height / regionAspect;
  }
  return { width: Math.round(width), height: Math.round(height) };
}

export function captureTargetPx(
  look: StillLook,
  windowFit: number,
  opts: { fit?: number; zoom?: Zoom; split?: number; aspect?: number } = {},
): number {
  const pieces = opts.split ?? 1;
  if (look === "cutout") return maxWindowPx(opts.fit ?? CUTOUT_FIT) * pieces;
  if (look === "bleed") {
    const zoom = opts.zoom ?? BLEED_ZOOM;
    if (zoom !== "fill") return maxWindowPx(windowFit * zoom) * pieces;
    if (!opts.aspect) return maxWindowPx(windowFit * BLEED_ZOOM) * pieces;
    return Math.round(
      Math.max(
        ...STILL_PRESET_IDS.map((id) => {
          const canvas = {
            width: STILL_PRESETS[id].width * pieces,
            height: STILL_PRESETS[id].height,
          };
          const base = windowBox(opts.aspect!, canvas, windowFit);
          const z = fillZoom(base, canvas);
          return Math.max(base.width, base.height) * z;
        }),
      ),
    );
  }
  return maxWindowPx(windowFit) * pieces;
}

export function anchorBox(
  size: Canvas,
  canvas: Canvas,
  anchor: Anchor,
  inset: number,
): { left: number; top: number } {
  const [v, h] = anchorAxes(anchor);
  const left =
    h === "left"
      ? canvas.width * inset
      : h === "right"
        ? canvas.width - canvas.width * inset - size.width
        : (canvas.width - size.width) / 2;
  const top =
    v === "top"
      ? canvas.height * inset
      : v === "bottom"
        ? canvas.height - canvas.height * inset - size.height
        : (canvas.height - size.height) / 2;
  return { left, top };
}

const anchorAxes = (anchor: Anchor): ["top" | "center" | "bottom", "left" | "center" | "right"] => {
  const v = anchor.startsWith("top") ? "top" : anchor.startsWith("bottom") ? "bottom" : "center";
  const h = anchor.endsWith("left") ? "left" : anchor.endsWith("right") ? "right" : "center";
  return [v, h];
};

export type Split = {
  count: number;
  piece: number;
  gutter: number;
};

export const MAX_SPLIT = 8;

export function splitCanvas(piece: Canvas, split: Split): { canvas: Canvas; offset: number } {
  const { count, gutter } = split;
  return {
    canvas: { width: piece.width * count + gutter * (count - 1), height: piece.height },
    offset: split.piece * (piece.width + gutter),
  };
}

export function splitProblem(v: { split?: unknown; gutter?: unknown }): string | null {
  if (
    v.split !== undefined &&
    !(Number.isInteger(v.split) && (v.split as number) >= 1 && (v.split as number) <= MAX_SPLIT)
  )
    return `\`split\` must be a whole number from 1 to ${MAX_SPLIT}`;
  if (v.gutter !== undefined && !(typeof v.gutter === "number" && v.gutter >= 0))
    return "`gutter` must be a number >= 0";
  if (v.gutter !== undefined && v.split === undefined) return "`gutter` needs a `split`";
  return null;
}

export function hasOwnShadow(meta: ShotMeta): boolean {
  const b = meta.body;
  if (!b) return false;
  const reach = Math.max(
    b.x,
    b.y,
    meta.region.width - (b.x + b.width),
    meta.region.height - (b.y + b.height),
  );
  return reach >= 3;
}

export type CutoutLayout = {
  img: Box;
  body: Box;
  scale: number;
  native: boolean;
  stretched: boolean;
  overflows: boolean;
};

export function cutoutLayout(
  meta: ShotMeta,
  canvas: Canvas,
  opts: { fit?: number; anchor?: Anchor; inset?: number; upscale?: number } = {},
): CutoutLayout {
  const body = meta.body;
  if (!body) throw new Error(`shot "${meta.name}" is not a cutout (no body)`);
  const fit = opts.fit ?? meta.fit ?? CUTOUT_FIT;
  const anchor = opts.anchor ?? meta.anchor ?? "center";
  const inset = opts.inset ?? meta.inset ?? 0;

  const png = shotPixels(meta);
  const bodyPx = { width: body.width * meta.scale, height: body.height * meta.scale };
  const want = windowBox(body.height / body.width, canvas, fit);
  const ceiling = opts.upscale ?? meta.upscale ?? 1;
  const scale = Math.min(ceiling, want.width / bodyPx.width);

  const placed = { width: bodyPx.width * scale, height: bodyPx.height * scale };
  const at = anchorBox(placed, canvas, anchor, inset);
  const left = Math.round(at.left - body.x * meta.scale * scale);
  const top = Math.round(at.top - body.y * meta.scale * scale);
  const img = {
    x: left,
    y: top,
    width: scale === 1 ? png.width : png.width * scale,
    height: scale === 1 ? png.height : png.height * scale,
  };
  return {
    img,
    body: {
      x: left + body.x * meta.scale * scale,
      y: top + body.y * meta.scale * scale,
      ...placed,
    },
    scale,
    native: scale === 1 && want.width / bodyPx.width >= 1,
    stretched: scale > 1,
    overflows:
      img.x < 0 ||
      img.y < 0 ||
      img.x + img.width > canvas.width ||
      img.y + img.height > canvas.height,
  };
}

export function bleedLayout(
  meta: ShotMeta,
  canvas: Canvas,
  windowFit: number,
  opts: { zoom?: Zoom; anchor?: Anchor; inset?: number } = {},
): Box {
  const asked = opts.zoom ?? meta.zoom ?? BLEED_ZOOM;
  const anchor = opts.anchor ?? meta.anchor ?? "top-left";
  const inset = opts.inset ?? meta.inset ?? BLEED_INSET;
  const base = windowBox(shotAspect(meta), canvas, windowFit);
  const zoom = asked === "fill" ? fillZoom(base, canvas) : asked;
  const size = {
    width: Math.round(base.width * zoom),
    height: Math.round(base.height * zoom),
  };
  const at = anchorBox(size, canvas, anchor, inset);
  return { x: Math.round(at.left), y: Math.round(at.top), ...size };
}

export function fallbackShotMeta(name: string): ShotMeta {
  return {
    name,
    region: { width: 1920, height: 1080 },
    scale: 2,
    viewport: { width: 1920, height: 1080 },
  };
}

export function shotMetaProblem(value: unknown): string | null {
  if (typeof value !== "object" || value === null) return "not an object";
  const meta = value as Partial<ShotMeta>;
  if (typeof meta.name !== "string" || meta.name === "") return "missing a `name`";
  for (const key of ["region", "viewport"] as const) {
    const box = meta[key];
    if (typeof box !== "object" || box === null) return `\`${key}\` must be an object`;
    if (!(typeof box.width === "number" && box.width > 0))
      return `\`${key}.width\` must be a number > 0`;
    if (!(typeof box.height === "number" && box.height > 0))
      return `\`${key}.height\` must be a number > 0`;
  }
  if (!(typeof meta.scale === "number" && meta.scale > 0)) return "`scale` must be a number > 0";
  const framing = framingProblem(meta) ?? splitProblem(meta);
  if (framing) return framing;
  if (meta.look === "cutout" && meta.body === undefined)
    return "a cutout sidecar must carry a `body` — re-run `pnpm shot`";
  return null;
}

export function framingProblem(v: {
  look?: unknown;
  fit?: unknown;
  anchor?: unknown;
  inset?: unknown;
  zoom?: unknown;
  shadow?: unknown;
  upscale?: unknown;
  ground?: unknown;
  body?: unknown;
}): string | null {
  if (v.look !== undefined && !isLook(v.look))
    return `\`look\` must be one of ${STILL_LOOKS.join(", ")}`;
  if (v.fit !== undefined && !(typeof v.fit === "number" && v.fit > 0 && v.fit <= MAX_CUTOUT_FIT))
    return `\`fit\` must be a number in (0, ${MAX_CUTOUT_FIT}]`;
  if (v.anchor !== undefined && !isAnchor(v.anchor))
    return `\`anchor\` must be one of ${ANCHORS.join(", ")}`;
  if (v.inset !== undefined && !(typeof v.inset === "number" && v.inset >= 0 && v.inset < 0.5))
    return "`inset` must be a number in [0, 0.5)";
  if (v.zoom !== undefined && !isZoom(v.zoom)) return '`zoom` must be a number >= 1, or "fill"';
  if (v.shadow !== undefined && typeof v.shadow !== "boolean") return "`shadow` must be a boolean";
  if (
    v.upscale !== undefined &&
    !(typeof v.upscale === "number" && v.upscale >= 1 && v.upscale <= MAX_UPSCALE)
  )
    return `\`upscale\` must be a number from 1 to ${MAX_UPSCALE}`;
  if (
    v.ground !== undefined &&
    !(
      typeof v.ground === "string" &&
      (v.ground === "transparent" || /^#[0-9a-f]{6}$/i.test(v.ground))
    )
  )
    return '`ground` must be a hex colour like #f3f3f3, or "transparent"';
  if (v.body !== undefined) {
    const b = v.body as Partial<Box>;
    if (typeof b !== "object" || b === null) return "`body` must be an object";
    for (const key of ["x", "y", "width", "height"] as const)
      if (typeof b[key] !== "number") return `\`body.${key}\` must be a number`;
    if (!(b.width! > 0 && b.height! > 0)) return "`body` must have a positive size";
  }
  return null;
}
