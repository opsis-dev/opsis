import { formatColor, parseColor } from "../../shared/color";
import { round3 } from "../../shared/math";
import type { NodeParams } from "../../shared/nodes/define";
import type { gifNode, scalePadNode } from "../../shared/nodes/media";
import { EDIT_VIDEO, evenDown, withEvenFrame } from "./codecs";

export function ffmpegColor(color: string): string {
  const parsed = parseColor(color.trim());
  return parsed ? formatColor(parsed, "hex") : "black";
}

const padTo = (w: number, h: number, color?: string) =>
  `scale=${w}:${h}:force_original_aspect_ratio=decrease,pad=${w}:${h}:(ow-iw)/2:(oh-ih)/2${color ? `:color=${color}` : ""},setsar=1`;

export function fitFilter({
  mode,
  width: w,
  height: h,
  background,
}: Pick<NodeParams<typeof scalePadNode>, "mode" | "width" | "height" | "background">): string {
  if (mode === "pad") return padTo(w, h, background ? ffmpegColor(background) : undefined);
  if (mode === "crop")
    return `scale=${w}:${h}:force_original_aspect_ratio=increase,crop=${w}:${h},setsar=1`;
  return `scale=${w}:${h},setsar=1`;
}

export const gifFilter = ({
  fps,
  width,
  dither,
}: Pick<NodeParams<typeof gifNode>, "fps" | "width" | "dither">): string =>
  `fps=${fps},scale=${width}:-1:flags=lanczos,split[a][b];[a]palettegen=stats_mode=diff[pal];[b][pal]paletteuse=dither=${dither}`;

export type JoinPart = { video: boolean; audio: boolean; durationMs: number };

const JOIN_AUDIO = "aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo";

export function joinFilter(
  parts: readonly JoinPart[],
  to: { width: number; height: number; fps: number },
): { filter: string; video: boolean; audio: boolean } {
  const rate = round3(to.fps);
  const width = evenDown(to.width);
  const height = evenDown(to.height);
  const video = parts.some((p) => p.video);
  const audio = parts.some((p) => p.audio);
  const chains: string[] = [];
  const pads: string[] = [];
  parts.forEach((p, i) => {
    const seconds = round3(p.durationMs / 1000);
    if (video) {
      chains.push(
        p.video
          ? `[${i}:v]${padTo(width, height)},fps=${rate},format=yuv420p[v${i}]`
          : `color=c=black:s=${width}x${height}:r=${rate}:d=${seconds},format=yuv420p[v${i}]`,
      );
      pads.push(`[v${i}]`);
    }
    if (audio) {
      chains.push(
        p.audio
          ? `[${i}:a]${JOIN_AUDIO}[a${i}]`
          : `anullsrc=r=48000:cl=stereo,atrim=duration=${seconds}[a${i}]`,
      );
      pads.push(`[a${i}]`);
    }
  });
  const out = `${video ? "[v]" : ""}${audio ? "[a]" : ""}`;
  const join = `${pads.join("")}concat=n=${parts.length}:v=${video ? 1 : 0}:a=${audio ? 1 : 0}${out}`;
  return { filter: `${chains.join(";")};${join}`, video, audio };
}

export function atempoChain(rate: number): string {
  const parts: number[] = [];
  let r = rate;
  while (r > 2) {
    parts.push(2);
    r /= 2;
  }
  while (r < 0.5) {
    parts.push(0.5);
    r /= 0.5;
  }
  parts.push(r);
  return parts.map((x) => `atempo=${x.toFixed(4)}`).join(",");
}

export function speedArgs(
  inPath: string,
  outPath: string,
  opts: { rate: number; fps: number; sound: boolean },
): string[] {
  return [
    "-i",
    inPath,
    "-vf",
    withEvenFrame(`setpts=PTS/${opts.rate}`),
    "-r",
    String(round3(opts.fps)),
    ...(opts.sound ? ["-af", atempoChain(opts.rate)] : ["-an"]),
    ...EDIT_VIDEO,
    outPath,
  ];
}
