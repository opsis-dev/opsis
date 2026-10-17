import { round3 } from "../../shared/math";
import { EDIT_VIDEO } from "./codecs";

export function joinFilter(
  count: number,
  to: { width: number; height: number; fps: number; sound: boolean },
): string {
  const { width: w, height: h } = to;
  const rate = round3(to.fps);
  const scaled = Array.from(
    { length: count },
    (_, i) =>
      `[${i}:v]scale=${w}:${h}:force_original_aspect_ratio=decrease,pad=${w}:${h}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps=${rate}[v${i}]`,
  ).join(";");
  const chain = Array.from(
    { length: count },
    (_, i) => `[v${i}]${to.sound ? `[${i}:a]` : ""}`,
  ).join("");
  return `${scaled};${chain}concat=n=${count}:v=1:a=${to.sound ? 1 : 0}[v]${to.sound ? "[a]" : ""}`;
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
    `setpts=PTS/${opts.rate}`,
    "-r",
    String(round3(opts.fps)),
    ...(opts.sound ? ["-af", atempoChain(opts.rate)] : ["-an"]),
    ...EDIT_VIDEO,
    outPath,
  ];
}
