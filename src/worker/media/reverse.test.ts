import { describe, expect, it } from "vitest";
import {
  frameBytes,
  joinReversedArgs,
  pieceLengthMs,
  REVERSE_BUDGET_BYTES,
  reverseArgs,
  reversedPartsList,
  reversePieces,
} from "./reverse";

const HD = { width: 1920, height: 1080 };
const HD_FRAME = frameBytes({ ...HD, pixFmt: "yuv420p" });

const piecesFor = (durationMs: number, fps = 30, bytes = HD_FRAME) =>
  reversePieces({ durationMs, fps, frameBytes: bytes, budgetBytes: REVERSE_BUDGET_BYTES });

const frameOf = (ms: number, fps: number) => (ms * fps) / 1000;

describe("frameBytes", () => {
  it.each([
    ["yuv420p", 1.5],
    ["yuvj420p", 1.5],
    ["nv12", 1.5],
    ["yuv422p", 2],
    ["yuv444p", 3],
    ["rgb24", 3],
    ["bgra", 4],
    ["yuv420p10le", 3],
    ["p010le", 3],
    ["yuv444p12le", 6],
    ["rgb48le", 6],
    ["rgba64le", 8],
    ["gray", 1],
    ["xyz12le", 4],
    ["", 4],
  ])("counts %s as %f bytes per pixel", (pixFmt, perPixel) => {
    expect(frameBytes({ width: 100, height: 10, pixFmt })).toBe(1000 * perPixel);
  });
});

describe("reversePieces", () => {
  it("keeps a short file in one piece", () => {
    expect(piecesFor(5000)).toEqual([{ from: undefined, to: undefined }]);
  });

  it("covers the whole file with no gap or overlap, last piece first", () => {
    const pieces = piecesFor(60_000);
    expect(pieces.length).toBeGreaterThan(1);
    expect(pieces[0].to).toBeUndefined();
    expect(pieces.at(-1)?.from).toBeUndefined();
    for (let n = 0; n + 1 < pieces.length; n++) {
      expect(pieces[n + 1].to).toBe(pieces[n].from);
    }
  });

  it("cuts the picture halfway between two frames and the sound at the next frame", () => {
    for (const fps of [30, 30000 / 1001, 60, 120]) {
      for (const { from } of piecesFor(120_000, fps).slice(0, -1)) {
        if (!from) throw new Error("Only the first piece starts at the beginning.");
        expect(Number.isInteger(from.pictureMs) && Number.isInteger(from.soundMs)).toBe(true);
        expect(Math.abs((frameOf(from.pictureMs, fps) % 1) - 0.5)).toBeLessThan(0.1);
        expect(
          Math.abs(frameOf(from.soundMs, fps) - Math.round(frameOf(from.soundMs, fps))),
        ).toBeLessThan(0.1);
      }
    }
  });

  it("holds the budget's worth of frames in each piece", () => {
    const [, middle] = piecesFor(60_000);
    const frames = frameOf(pieceLengthMs(middle, 60_000), 30);
    expect(Math.round(frames)).toBe(Math.floor(REVERSE_BUDGET_BYTES / HD_FRAME));
  });

  it("gives shorter pieces for 4K and for 10-bit", () => {
    const length = (bytes: number) => pieceLengthMs(piecesFor(60_000, 30, bytes)[1], 60_000);
    const hd = length(HD_FRAME);
    expect(length(frameBytes({ width: 3840, height: 2160, pixFmt: "yuv420p" }))).toBeLessThan(hd);
    expect(length(frameBytes({ ...HD, pixFmt: "yuv420p10le" }))).toBeLessThan(hd);
  });

  it("keeps at least one frame in a piece when a frame is over the budget", () => {
    const pieces = piecesFor(1000, 30, REVERSE_BUDGET_BYTES * 2);
    expect(pieces).toHaveLength(30);
    expect(Math.abs(pieceLengthMs(pieces[1], 1000) - 1000 / 30)).toBeLessThanOrEqual(1);
  });
});

describe("reverseArgs", () => {
  it("reverses a whole file with today's single command", () => {
    expect(reverseArgs("in.mp4", "out.mp4", { sound: true })).toEqual([
      "-i",
      "in.mp4",
      "-vf",
      "reverse,crop=trunc(iw/2)*2:trunc(ih/2)*2",
      "-af",
      "areverse",
      "-c:v",
      "libx264",
      "-crf",
      "18",
      "-pix_fmt",
      "yuv420p",
      "out.mp4",
    ]);
  });

  it("cuts the picture and the sound of a piece at their own times, from zero", () => {
    const args = reverseArgs("in.mp4", "part.mkv", {
      piece: { from: { pictureMs: 1983, soundMs: 2000 }, to: { pictureMs: 3983, soundMs: 4000 } },
      sound: true,
      soundCodec: ["-c:a", "pcm_s16le"],
    });
    expect(args.slice(0, 8)).toEqual([
      "-ss",
      "1.983",
      "-i",
      "in.mp4",
      "-vf",
      "trim=end=2.000,reverse,setpts=PTS-STARTPTS,crop=trunc(iw/2)*2:trunc(ih/2)*2",
      "-af",
      "atrim=start=0.017:end=2.017,areverse,asetpts=PTS-STARTPTS",
    ]);
    expect(args).toContain("pcm_s16le");
  });

  it("reads the first piece from the start and the last piece to the end", () => {
    const cut = { pictureMs: 483, soundMs: 500 };
    const first = reverseArgs("in.mp4", "p.mkv", { piece: { to: cut }, sound: true });
    expect(first.slice(0, 6)).toEqual([
      "-i",
      "in.mp4",
      "-vf",
      "trim=end=0.483,reverse,crop=trunc(iw/2)*2:trunc(ih/2)*2",
      "-af",
      "atrim=end=0.500,areverse",
    ]);
    const last = reverseArgs("in.mp4", "p.mkv", { piece: { from: cut }, sound: true });
    expect(last.slice(0, 8)).toEqual([
      "-ss",
      "0.483",
      "-i",
      "in.mp4",
      "-vf",
      "reverse,setpts=PTS-STARTPTS,crop=trunc(iw/2)*2:trunc(ih/2)*2",
      "-af",
      "atrim=start=0.017,areverse,asetpts=PTS-STARTPTS",
    ]);
  });

  it("drops the sound when there is none", () => {
    expect(reverseArgs("in.mp4", "out.mp4", { sound: false })).toContain("-an");
  });
});

describe("reversedPartsList", () => {
  it("gives every part its exact length so the joins leave no gap", () => {
    expect(
      reversedPartsList([
        { file: "/a/part-0.mkv", lengthMs: 2800 },
        { file: "/a/it's/part-1.mkv", lengthMs: 5733 },
      ]),
    ).toBe("file '/a/part-0.mkv'\nduration 2.800\nfile '/a/it'\\''s/part-1.mkv'\nduration 5.733");
  });
});

describe("joinReversedArgs", () => {
  it("copies the picture and encodes the sound once", () => {
    expect(joinReversedArgs("parts.txt", "out.mp4", true)).toEqual([
      "-f",
      "concat",
      "-safe",
      "0",
      "-i",
      "parts.txt",
      "-map",
      "0",
      "-c:v",
      "copy",
      "-c:a",
      "aac",
      "out.mp4",
    ]);
    expect(joinReversedArgs("parts.txt", "out.mp4", false)).not.toContain("-c:a");
  });
});
