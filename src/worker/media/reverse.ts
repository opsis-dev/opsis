import { singleQuoted } from "@opsis/core/format";
import { EDIT_AUDIO, EDIT_VIDEO, ffmpegSeconds, withEvenFrame } from "./codecs";

export const REVERSE_BUDGET_BYTES = 512 * 1024 ** 2;

const UNKNOWN_BYTES_PER_PIXEL = 4;

const BYTES_PER_PIXEL: ReadonlyArray<readonly [RegExp, number]> = [
  [/^gray/, 1],
  [/^(?:yuvj?420p|nv12|nv21|p01[026])/, 1.5],
  [/^(?:yuvj?422p|nv16|yuyv422|uyvy422|p21[026])/, 2],
  [/^(?:rgb|bgr)48/, 6],
  [/^(?:rgb|bgr)a64/, 8],
  [/^(?:yuvj?444p|gbrp|rgb24|bgr24|p41[026])/, 3],
  [/^(?:rgba|bgra|argb|abgr|rgb0|bgr0|0rgb|0bgr)$/, 4],
];

const HIGH_DEPTH = /(?:p|gray)(?:9|10|12|14|16)(?:le|be)?$|^p[024]1[026]/;

type FrameShape = { width: number; height: number; pixFmt: string };

export function frameBytes({ width, height, pixFmt }: FrameShape): number {
  const perPixel =
    BYTES_PER_PIXEL.find(([pattern]) => pattern.test(pixFmt))?.[1] ?? UNKNOWN_BYTES_PER_PIXEL;
  return Math.ceil(width * height * perPixel * (HIGH_DEPTH.test(pixFmt) ? 2 : 1));
}

export type ReverseCut = { pictureMs: number; soundMs: number };

export type ReversePiece = { from?: ReverseCut; to?: ReverseCut };

export const WHOLE_FILE: ReversePiece = {};

type PieceBudget = { durationMs: number; fps: number; frameBytes: number; budgetBytes: number };

const cutBefore = (frame: number, fps: number): ReverseCut => ({
  pictureMs: Math.round(((frame - 0.5) * 1000) / fps),
  soundMs: Math.round((frame * 1000) / fps),
});

export function reversePieces({
  durationMs,
  fps,
  frameBytes: bytes,
  budgetBytes,
}: PieceBudget): ReversePiece[] {
  const framesPerPiece = Math.max(1, Math.floor(budgetBytes / bytes));
  const frames = Math.ceil((durationMs * fps) / 1000);
  const cuts: Array<ReverseCut | undefined> = [undefined];
  for (let first = framesPerPiece; first < frames; first += framesPerPiece) {
    cuts.push(cutBefore(first, fps));
  }
  return cuts.map((from, n) => ({ from, to: cuts[n + 1] })).toReversed();
}

export const pieceLengthMs = ({ from, to }: ReversePiece, durationMs: number): number =>
  Math.max(0, (to?.soundMs ?? durationMs) - (from?.soundMs ?? 0));

const trimFilter = (name: string, startMs: number, endMs: number | undefined) => {
  const bounds = [
    ...(startMs ? [`start=${ffmpegSeconds(startMs)}`] : []),
    ...(endMs === undefined ? [] : [`end=${ffmpegSeconds(endMs)}`]),
  ];
  return bounds.length ? [`${name}=${bounds.join(":")}`] : [];
};

const pictureFilter = ({ from, to }: ReversePiece): string => {
  const seekMs = from?.pictureMs ?? 0;
  return [
    ...trimFilter("trim", 0, to && to.pictureMs - seekMs),
    "reverse",
    ...(from ? ["setpts=PTS-STARTPTS"] : []),
  ].join(",");
};

const soundFilter = ({ from, to }: ReversePiece): string => {
  const seekMs = from?.pictureMs ?? 0;
  return [
    ...trimFilter("atrim", from ? from.soundMs - seekMs : 0, to && to.soundMs - seekMs),
    "areverse",
    ...(from ? ["asetpts=PTS-STARTPTS"] : []),
  ].join(",");
};

type ReverseOptions = { piece?: ReversePiece; sound: boolean; soundCodec?: readonly string[] };

export const reverseArgs = (
  inPath: string,
  outPath: string,
  { piece = WHOLE_FILE, sound, soundCodec = [] }: ReverseOptions,
): string[] => [
  ...(piece.from ? ["-ss", ffmpegSeconds(piece.from.pictureMs)] : []),
  "-i",
  inPath,
  "-vf",
  withEvenFrame(pictureFilter(piece)),
  ...(sound ? ["-af", soundFilter(piece), ...soundCodec] : ["-an"]),
  ...EDIT_VIDEO,
  outPath,
];

export const reverseSoundArgs = (
  inPath: string,
  outPath: string,
  codecs: readonly string[],
): string[] => ["-i", inPath, "-af", "areverse", ...codecs, outPath];

type ReversedPart = { file: string; lengthMs: number };

export const reversedPartsList = (parts: readonly ReversedPart[]): string =>
  parts
    .map(({ file, lengthMs }) => `file ${singleQuoted(file)}\nduration ${ffmpegSeconds(lengthMs)}`)
    .join("\n");

export const joinReversedArgs = (listPath: string, outPath: string, sound: boolean): string[] => [
  "-f",
  "concat",
  "-safe",
  "0",
  "-i",
  listPath,
  "-map",
  "0",
  "-c:v",
  "copy",
  ...(sound ? EDIT_AUDIO : []),
  outPath,
];
