import { round3 } from "../../shared/math";
import type { NodeParams } from "../../shared/nodes/define";
import type { gifNode, scalePadNode } from "../../shared/nodes/media";
import { EDIT_VIDEO, evenDown, withEvenFrame } from "./codecs";

const padTo = (w: number, h: number, color?: string) =>
  `scale=${w}:${h}:force_original_aspect_ratio=decrease,pad=${w}:${h}:(ow-iw)/2:(oh-ih)/2${color ? `:color=${color}` : ""},setsar=1`;

export function fitFilter({
  mode,
  width: w,
  height: h,
  background,
}: Pick<NodeParams<typeof scalePadNode>, "mode" | "width" | "height" | "background">): string {
  if (mode === "pad") return padTo(w, h, background);
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

export function joinFilter(
  count: number,
  to: { width: number; height: number; fps: number; sound: boolean },
): string {
  const rate = round3(to.fps);
  const width = evenDown(to.width);
  const height = evenDown(to.height);
  const inputs = Array.from({ length: count }, (_, i) => i);
  const scaled = inputs.map((i) => `[${i}:v]${padTo(width, height)},fps=${rate}[v${i}]`);
  const join = to.sound
    ? { pads: inputs.map((i) => `[v${i}][${i}:a]`), audio: 1, out: "[v][a]" }
    : { pads: inputs.map((i) => `[v${i}]`), audio: 0, out: "[v]" };
  return `${scaled.join(";")};${join.pads.join("")}concat=n=${count}:v=1:a=${join.audio}${join.out}`;
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
