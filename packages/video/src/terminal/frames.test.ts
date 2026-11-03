import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { decodeFrames, frameAt, hexColour } from "#video/terminal/frames.ts";

const FIXTURES = path.resolve(import.meta.dirname, "../../../../fixtures/terminal");

type Summary = {
  size: [number, number];
  fps: number;
  frames: number;
  rows: number;
  colours: string[];
  last: string[];
};

function lineOf(runs: Array<{ col: number; cells: number; text: string }>): string {
  let line = "";
  let at = 0;
  for (const run of runs) {
    line += " ".repeat(Math.max(0, run.col - at)) + run.text;
    at = run.col + run.cells;
  }
  return line.trimEnd();
}

describe("terminal frames", () => {
  const frames = decodeFrames(readFileSync(path.join(FIXTURES, "take.frames")));
  const summary = JSON.parse(
    readFileSync(path.join(FIXTURES, "take.summary.json"), "utf8"),
  ) as Summary;

  it("reads the frames file Rust wrote as Rust summed it up", () => {
    expect([frames.cols, frames.rows]).toEqual(summary.size);
    expect(frames.fps).toBe(summary.fps);
    expect(frames.frames.length).toBe(summary.frames);
    expect(frames.rowTable.length).toBe(summary.rows);
    const last = frames.frames.at(-1);
    if (!last) throw new Error("no frames");
    expect(last.rowIndices.map((i) => lineOf(frames.rowTable[i] ?? []))).toEqual(summary.last);
    const colours = new Set(
      frames.rowTable.flat().flatMap((r) => [hexColour(r.fg), hexColour(r.bg)]),
    );
    expect([...colours].sort()).toEqual(summary.colours);
  });

  it("picks the frame of a time and holds the ends", () => {
    expect(frameAt(frames, -1)).toBe(frames.frames[0]);
    expect(frameAt(frames, 1)).toBe(frames.frames[frames.fps]);
    expect(frameAt(frames, 1e6)).toBe(frames.frames.at(-1));
  });

  it("refuses a file of another kind", () => {
    expect(() => decodeFrames(new TextEncoder().encode("NOTFRAME"))).toThrow(
      "This isn't a terminal frames file.",
    );
  });
});
