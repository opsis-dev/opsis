import { useCallback, useEffect, useMemo, useRef, useSyncExternalStore } from "react";
import { clamp, numberOr } from "@/shared/math";
import type { Filmstrip, Waveform, WaveformLevel } from "@/shared/analysis";
import type { AudioBar, VideoBar } from "@/shared/timeline/flatten";
import { cn } from "@/renderer/lib/utils";
import { useMedia } from "../../store/media";
import { fileUrl } from "../file-url";
import { OFFSCREEN_PX, type Viewport } from "./timeline-layout";
import { isProjectRef } from "@/shared/asset-ref";

const END_EPSILON = 0.001;
const DISPLAY_RANGE_DB = 48;
const SILENT_AMPLITUDE = 1e-4;
const WAVE_FILL = 0.95;
const MAX_WAVE_GAIN = 2;

type Visible = { offset: number; width: number };

function visiblePart(barLeft: number, barWidth: number, view: Viewport): Visible | null {
  const a = Math.max(barLeft, view.left - OFFSCREEN_PX);
  const b = Math.min(barLeft + barWidth, view.left + view.width + OFFSCREEN_PX);
  if (b - a < 1) return null;
  return { offset: a - barLeft, width: Math.ceil(b - a) };
}

const images = new Map<string, HTMLImageElement>();

function cachedImage(ref: string): HTMLImageElement {
  let img = images.get(ref);
  if (!img) {
    img = new Image();
    img.src = fileUrl(ref);
    images.set(ref, img);
  }
  return img;
}

function useImage(ref: string | null): HTMLImageElement | null {
  const img = ref ? cachedImage(ref) : null;
  const subscribe = useCallback(
    (onLoad: () => void) => {
      img?.addEventListener("load", onLoad);
      return () => img?.removeEventListener("load", onLoad);
    },
    [img],
  );
  return useSyncExternalStore(subscribe, () =>
    img?.complete && img.naturalWidth > 0 ? img : null,
  );
}

function prepare(canvas: HTMLCanvasElement, width: number, height: number) {
  const dpr = window.devicePixelRatio || 1;
  canvas.width = Math.max(1, Math.round(width * dpr));
  canvas.height = Math.max(1, Math.round(height * dpr));
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);
  return ctx;
}

type Draw = (ctx: CanvasRenderingContext2D, part: Visible) => void;

function useBarCanvas(
  left: number,
  width: number,
  height: number,
  view: Viewport,
  draw: Draw | null,
) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const part = visiblePart(left, width, view);
  const offset = part?.offset ?? 0;
  const w = part?.width ?? 0;
  useEffect(() => {
    if (!canvas.current || !draw || w < 1) return;
    const ctx = prepare(canvas.current, w, height);
    if (ctx) draw(ctx, { offset, width: w });
  }, [draw, offset, w, height]);
  return { canvas, part };
}

function BarCanvas({
  canvas,
  part,
  height,
  className,
}: {
  canvas: React.RefObject<HTMLCanvasElement | null>;
  part: Visible;
  height: number;
  className?: string;
}) {
  return (
    <canvas
      ref={canvas}
      className={cn("pointer-events-none absolute top-0", className)}
      style={{ left: part.offset, width: part.width, height }}
    />
  );
}

const tileStart = (left: number, offset: number, tileW: number) =>
  Math.floor((left + offset) / tileW) * tileW - left;

function stringProp(props: Record<string, unknown>, key: string): string | null {
  const v = props[key];
  return typeof v === "string" && v ? v : null;
}

type BarSource =
  | { kind: "filmstrip"; src: string; sourceMs: (local: number, fps: number) => number }
  | { kind: "still"; src: string };

function barSource(bar: VideoBar): BarSource | null {
  const p = bar.props;
  switch (bar.component) {
    case "core/recording": {
      const src = stringProp(p, "proxySrc") ?? stringProp(p, "src");
      const speed = numberOr(p.speed, 1) || 1;
      return src
        ? { kind: "filmstrip", src, sourceMs: (local, fps) => (local / fps) * speed * 1000 }
        : null;
    }
    case "core/video": {
      const src = stringProp(p, "src");
      const trimMs = numberOr(p.trimBeforeMs, 0);
      return src
        ? { kind: "filmstrip", src, sourceMs: (local, fps) => (local / fps) * 1000 + trimMs }
        : null;
    }
    case "core/image":
    case "core/still": {
      const src = stringProp(p, "src");
      return src && isProjectRef(src) ? { kind: "still", src } : null;
    }
    default:
      return null;
  }
}

type SpriteGeometry = {
  bar: VideoBar;
  left: number;
  height: number;
  pxPerFrame: number;
  fps: number;
  sourceMs: (local: number, fps: number) => number;
};

function drawSprite(
  ctx: CanvasRenderingContext2D,
  part: Visible,
  strip: Filmstrip,
  img: HTMLImageElement,
  g: SpriteGeometry,
) {
  const tileW = (strip.tileWidth / strip.tileHeight) * g.height;
  for (let x = tileStart(g.left, part.offset, tileW); x < part.offset + part.width; x += tileW) {
    const abs = (g.left + x + tileW / 2) / g.pxPerFrame;
    const at = clamp(abs, g.bar.from, g.bar.to - END_EPSILON);
    const ms = g.sourceMs(g.bar.localAt(at), g.fps);
    const index = clamp(Math.floor(ms / strip.intervalMs), 0, strip.count - 1);
    ctx.drawImage(
      img,
      index * strip.tileWidth,
      0,
      strip.tileWidth,
      strip.tileHeight,
      x - part.offset,
      0,
      tileW,
      g.height,
    );
  }
}

function drawStill(
  ctx: CanvasRenderingContext2D,
  part: Visible,
  img: HTMLImageElement,
  left: number,
  height: number,
) {
  const tileW = (img.naturalWidth / img.naturalHeight) * height;
  for (let x = tileStart(left, part.offset, tileW); x < part.offset + part.width; x += tileW)
    ctx.drawImage(img, x - part.offset, 0, tileW, height);
}

export function FilmstripCanvas({
  bar,
  left,
  width,
  height,
  view,
  fps,
}: {
  bar: VideoBar;
  left: number;
  width: number;
  height: number;
  view: Viewport;
  fps: number;
}) {
  const source = useMemo(() => barSource(bar), [bar]);
  const filmSrc = source?.kind === "filmstrip" ? source.src : null;
  const entry = useMedia((s) => (filmSrc ? s.filmstrips[filmSrc] : undefined));
  const strip = entry && typeof entry === "object" ? entry : null;
  useEffect(() => {
    if (filmSrc) useMedia.getState().filmstrip(filmSrc);
  }, [filmSrc]);
  const spriteImg = useImage(strip?.image ?? null);
  const stillImg = useImage(source?.kind === "still" ? source.src : null);
  const pxPerFrame = view.pxPerFrame;
  const draw = useCallback<Draw>(
    (ctx, part) => {
      if (source?.kind === "filmstrip" && strip && spriteImg)
        drawSprite(ctx, part, strip, spriteImg, {
          bar,
          left,
          height,
          pxPerFrame,
          fps,
          sourceMs: source.sourceMs,
        });
      else if (stillImg) drawStill(ctx, part, stillImg, left, height);
    },
    [source, strip, spriteImg, stillImg, bar, left, height, pxPerFrame, fps],
  );
  const { canvas, part } = useBarCanvas(left, width, height, view, draw);
  if (!part || (!strip && source?.kind !== "still")) return null;
  return <BarCanvas canvas={canvas} part={part} height={height} className="opacity-60" />;
}

function shape(v: number): number {
  const a = Math.min(1, Math.abs(v));
  if (a < SILENT_AMPLITUDE) return 0;
  return Math.sign(v) * Math.max(0, 1 + (20 * Math.log10(a)) / DISPLAY_RANGE_DB);
}

function pickLevel(waveform: Waveform, pxPerSecond: number): WaveformLevel | null {
  const levels = waveform.levels.toSorted((a, b) => a.rate - b.rate);
  return levels.find((l) => l.rate >= pxPerSecond) ?? levels.at(-1) ?? null;
}

type WaveGeometry = {
  bar: AudioBar;
  left: number;
  height: number;
  pxPerFrame: number;
  fps: number;
  gain: (abs: number) => number;
};

function drawWaveform(
  ctx: CanvasRenderingContext2D,
  part: Visible,
  wave: Waveform,
  g: WaveGeometry,
) {
  const level = pickLevel(wave, g.pxPerFrame * g.fps);
  if (!level) return;
  const { bar } = g;
  const durationS = wave.durationMs / 1000;
  const mid = g.height / 2;
  ctx.fillStyle = getComputedStyle(ctx.canvas).getPropertyValue("--waveform").trim() || "gray";
  for (let i = 0; i < part.width; i++) {
    const abs = (g.left + part.offset + i) / g.pxPerFrame;
    if (abs < bar.from || abs >= bar.to) continue;
    let t = (abs - bar.from + bar.trimBefore) / g.fps;
    if (bar.loop && durationS > 0) t %= durationS;
    const k = Math.floor(t * level.rate);
    if (k < 0 || k >= level.max.length) continue;
    const gain = Math.min(MAX_WAVE_GAIN, bar.volume * g.gain(abs));
    const top = mid - shape(level.max[k] * gain) * mid * WAVE_FILL;
    const bottom = mid - shape(level.min[k] * gain) * mid * WAVE_FILL;
    ctx.fillRect(i, top, 1, Math.max(1, bottom - top));
  }
}

export function WaveformCanvas({
  bar,
  left,
  width,
  height,
  view,
  fps,
  gain,
}: {
  bar: AudioBar;
  left: number;
  width: number;
  height: number;
  view: Viewport;
  fps: number;
  gain: (abs: number) => number;
}) {
  const entry = useMedia((s) => s.waveforms[bar.src]);
  const wave = entry && typeof entry === "object" ? entry : null;
  useEffect(() => {
    useMedia.getState().waveform(bar.src);
  }, [bar.src]);
  const pxPerFrame = view.pxPerFrame;
  const draw = useCallback<Draw>(
    (ctx, part) => {
      if (wave) drawWaveform(ctx, part, wave, { bar, left, height, pxPerFrame, fps, gain });
    },
    [wave, bar, left, height, pxPerFrame, fps, gain],
  );
  const { canvas, part } = useBarCanvas(left, width, height, view, draw);
  if (!part || !wave) return null;
  return <BarCanvas canvas={canvas} part={part} height={height} />;
}
