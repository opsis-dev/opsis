import { useCallback, useEffect, useRef, useSyncExternalStore } from "react";
import { clamp } from "@/shared/math";
import { lruCache } from "@/shared/memo";
import type { Filmstrip, Waveform, WaveformLevel } from "@/shared/analysis";
import { MAX_VOLUME } from "@/shared/nodes/params";
import type { AudioBar, BarThumbnail, VideoBar } from "@/shared/timeline/flatten";
import { cn } from "@/renderer/lib/utils";
import { loadedFilmstrip, loadedWaveform, useMedia } from "../../store/media";
import { fileUrl } from "../file-url";
import { OFFSCREEN_PX, type Viewport } from "./timeline-layout";

const END_EPSILON = 0.001;
const DISPLAY_RANGE_DB = 48;
const SILENT_AMPLITUDE = 1e-4;
const WAVE_FILL = 0.95;
const CACHED_IMAGES = 64;

type Visible = { offset: number; width: number };

function visiblePart(barLeft: number, barWidth: number, view: Viewport): Visible | null {
  const a = Math.max(barLeft, view.left - OFFSCREEN_PX);
  const b = Math.min(barLeft + barWidth, view.left + view.width + OFFSCREEN_PX);
  if (b - a < 1) return null;
  return { offset: a - barLeft, width: Math.ceil(b - a) };
}

const images = lruCache<HTMLImageElement>(CACHED_IMAGES);

function newImage(ref: string): HTMLImageElement {
  const img = new Image();
  img.src = fileUrl(ref);
  return img;
}

const isLoaded = (img: HTMLImageElement | undefined): img is HTMLImageElement =>
  !!img?.complete && img.naturalWidth > 0;

function useImage(ref: string | null): HTMLImageElement | null {
  const held = useRef<{ ref: string; img: HTMLImageElement } | null>(null);
  const subscribe = useCallback(
    (onLoad: () => void) => {
      if (!ref) return () => {};
      const img = images.use(ref, () => newImage(ref));
      held.current = { ref, img };
      img.addEventListener("load", onLoad);
      return () => img.removeEventListener("load", onLoad);
    },
    [ref],
  );
  return useSyncExternalStore(subscribe, () => {
    if (!ref) return null;
    const img = held.current?.ref === ref ? held.current.img : images.peek(ref);
    return isLoaded(img) ? img : null;
  });
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

type Filmed = Extract<BarThumbnail, { kind: "filmstrip" }>;

const sourceMs = (thumb: Filmed, local: number, fps: number): number =>
  (local / fps) * thumb.speed * 1000 + thumb.trimMs;

type SpriteGeometry = {
  bar: VideoBar;
  thumb: Filmed;
  left: number;
  height: number;
  pxPerFrame: number;
  fps: number;
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
    const ms = sourceMs(g.thumb, g.bar.localAt(at), g.fps);
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
  const source = bar.thumbnail;
  const filmSrc = source?.kind === "filmstrip" ? source.src : null;
  const strip = useMedia(loadedFilmstrip(filmSrc));
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
          thumb: source,
          left,
          height,
          pxPerFrame,
          fps,
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

export type Gain = (abs: number) => number;

type WaveGeometry = {
  bar: AudioBar;
  left: number;
  height: number;
  pxPerFrame: number;
  fps: number;
  gain: Gain;
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
    const gain = Math.min(MAX_VOLUME, bar.volume * g.gain(abs));
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
  gain: Gain;
}) {
  const wave = useMedia(loadedWaveform(bar.src));
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
