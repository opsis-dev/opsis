import { failRender, holdRender } from "@opsis/runtime/holds.ts";
import { type CSSProperties, useLayoutEffect, useRef } from "react";
import type { Fit } from "#video/generated/spec.ts";

const US_PER_S = 1_000_000;

export type FootageUrls = Partial<Record<string, string>>;

function footageFrameUrl(base: string, timeS: number): string {
  return `${base}${Math.max(0, Math.round(timeS * US_PER_S))}`;
}

async function failedFrame(src: string): Promise<Error> {
  try {
    const response = await fetch(src);
    const text = await response.text();
    return new Error(text || `Couldn't show a frame of a video (${response.status}).`);
  } catch (error) {
    return error instanceof Error ? error : new Error(String(error));
  }
}

export function FootageFrame({
  url,
  timeS,
  fit,
  style,
}: {
  url: string;
  timeS: number;
  fit: Fit;
  style?: CSSProperties;
}) {
  const image = useRef<HTMLImageElement>(null);
  const src = footageFrameUrl(url, timeS);
  useLayoutEffect(() => {
    const target = image.current;
    if (!target) return;
    const release = holdRender(`Showing ${src}`);
    let alive = true;
    target.decode().then(release, async () => {
      if (!alive) return;
      failRender(await failedFrame(src));
    });
    return () => {
      // oxlint-disable-next-line react/immutability
      alive = false;
      release();
    };
  }, [src]);
  return (
    <img
      ref={image}
      src={src}
      alt=""
      style={{ width: "100%", height: "100%", objectFit: fit, display: "block", ...style }}
    />
  );
}
