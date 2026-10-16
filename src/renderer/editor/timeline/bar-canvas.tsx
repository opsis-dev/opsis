import { useEffect, useRef, useState } from "react";
import type { Waveform } from "@/shared/mix/result";
import type { AudioBar, VideoBar } from "@/shared/timeline/flatten";
import { useMedia } from "../../store/media";
import { fileUrl } from "../file-url";

export type Viewport = { left: number; width: number; pxPerFrame: number };

type Visible = { offset: number; width: number };

export function visiblePart(barLeft: number, barWidth: number, view: Viewport): Visible | null {
  const a = Math.max(barLeft, view.left - 64);
  const b = Math.min(barLeft + barWidth, view.left + view.width + 64);
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
  const [, setLoaded] = useState(0);
  const img = ref ? cachedImage(ref) : null;
  useEffect(() => {
    if (!img || img.complete) return;
    const onLoad = () => setLoaded((n) => n + 1);
    img.addEventListener("load", onLoad);
    return () => img.removeEventListener("load", onLoad);
  }, [img]);
  return img && img.complete && img.naturalWidth > 0 ? img : null;
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

export function filmstripSource(bar: VideoBar): string | null {
  const p = bar.props;
  if (bar.component === "core/recording") return String(p.proxySrc ?? p.src ?? "") || null;
  if (bar.component === "core/video") return String(p.src ?? "") || null;
  return null;
}

export function imageSource(bar: VideoBar): string | null {
  if (bar.component !== "core/image" && bar.component !== "core/still") return null;
  const src = bar.props.src;
  return typeof src === "string" && src.startsWith("project:") ? src : null;
}

function sourceMs(bar: VideoBar, local: number, fps: number): number {
  if (bar.component === "core/recording") {
    const speed = Number(bar.props.speed ?? 1) || 1;
    return (local / fps) * speed * 1000;
  }
  return (local / fps) * 1000 + Number(bar.props.trimBeforeMs ?? 0);
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
  const src = filmstripSource(bar);
  const stillRef = imageSource(bar);
  const entry = useMedia((s) => (src ? s.filmstrips[src] : undefined));
  const strip = entry && typeof entry === "object" ? entry : null;
  useEffect(() => {
    if (src) useMedia.getState().filmstrip(src);
  }, [src]);
  const spriteImg = useImage(strip?.image ?? null);
  const stillImg = useImage(stillRef);
  const part = visiblePart(left, width, view);
  const canvas = useRef<HTMLCanvasElement>(null);
  const offset = part?.offset ?? 0;
  const w = part?.width ?? 0;

  useEffect(() => {
    if (!canvas.current || w < 1) return;
    const ctx = prepare(canvas.current, w, height);
    if (!ctx) return;
    if (strip && spriteImg) {
      const tileW = (strip.tileWidth / strip.tileHeight) * height;
      const start = Math.floor((left + offset) / tileW) * tileW - left;
      for (let x = start; x < offset + w; x += tileW) {
        const abs = (left + x + tileW / 2) / view.pxPerFrame;
        const clamped = Math.min(bar.to - 0.001, Math.max(bar.from, abs));
        const ms = sourceMs(bar, bar.localAt(clamped), fps);
        const index = Math.max(0, Math.min(strip.count - 1, Math.floor(ms / strip.intervalMs)));
        ctx.drawImage(
          spriteImg,
          index * strip.tileWidth,
          0,
          strip.tileWidth,
          strip.tileHeight,
          x - offset,
          0,
          tileW,
          height,
        );
      }
    } else if (stillImg) {
      const tileW = (stillImg.naturalWidth / stillImg.naturalHeight) * height;
      const start = Math.floor((left + offset) / tileW) * tileW - left;
      for (let x = start; x < offset + w; x += tileW)
        ctx.drawImage(stillImg, x - offset, 0, tileW, height);
    }
  }, [strip, spriteImg, stillImg, left, offset, w, height, view.pxPerFrame, bar, fps]);

  if (!part || (!strip && !stillRef)) return null;
  return (
    <canvas
      ref={canvas}
      className="pointer-events-none absolute top-0 opacity-60"
      style={{ left: offset, width: w, height }}
    />
  );
}

const DISPLAY_RANGE_DB = 48;

function shape(v: number): number {
  const a = Math.min(1, Math.abs(v));
  if (a < 1e-4) return 0;
  return Math.sign(v) * Math.max(0, 1 + (20 * Math.log10(a)) / DISPLAY_RANGE_DB);
}

function pickLevel(waveform: Waveform, pxPerSecond: number) {
  const levels = waveform.levels.toSorted((a, b) => a.rate - b.rate);
  return levels.find((l) => l.rate >= pxPerSecond) ?? levels[levels.length - 1];
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
  const part = visiblePart(left, width, view);
  const canvas = useRef<HTMLCanvasElement>(null);
  const offset = part?.offset ?? 0;
  const w = part?.width ?? 0;

  useEffect(() => {
    if (!canvas.current || !wave || w < 1) return;
    const ctx = prepare(canvas.current, w, height);
    if (!ctx) return;
    const level = pickLevel(wave, view.pxPerFrame * fps);
    const durationS = wave.durationMs / 1000;
    const mid = height / 2;
    ctx.fillStyle =
      getComputedStyle(canvas.current).getPropertyValue("--waveform").trim() || "gray";
    for (let i = 0; i < w; i++) {
      const abs = (left + offset + i) / view.pxPerFrame;
      if (abs < bar.from || abs >= bar.to) continue;
      let t = (abs - bar.from + bar.trimBefore) / fps;
      if (bar.loop && durationS > 0) t %= durationS;
      const k = Math.floor(t * level.rate);
      if (k < 0 || k >= level.max.length) continue;
      const g = Math.min(2, bar.volume * gain(abs));
      const hi = shape(level.max[k] * g);
      const lo = shape(level.min[k] * g);
      const top = mid - hi * mid * 0.95;
      const bottom = mid - lo * mid * 0.95;
      ctx.fillRect(i, top, 1, Math.max(1, bottom - top));
    }
  }, [wave, left, offset, w, height, view.pxPerFrame, bar, fps, gain]);

  if (!part || !wave) return null;
  return (
    <canvas
      ref={canvas}
      className="pointer-events-none absolute top-0"
      style={{ left: offset, width: w, height }}
    />
  );
}
