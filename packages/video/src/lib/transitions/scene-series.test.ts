import * as fc from "fast-check";
import { expect, it } from "vitest";
import { cutOverlaps, sceneStarts, type SeriesCut } from "#video/lib/transitions/scene-series.tsx";

function cutOf(durationInFrames: number): SeriesCut {
  return { timing: { durationInFrames, progressAt: () => 0 }, play: () => null };
}

const series = fc
  .array(fc.integer({ min: 61, max: 400 }), { minLength: 1, maxLength: 8 })
  .chain((lengths) =>
    fc.tuple(
      fc.constant(lengths),
      fc.array(fc.option(fc.integer({ min: 1, max: 30 }), { nil: null }), {
        minLength: lengths.length - 1,
        maxLength: lengths.length - 1,
      }),
    ),
  );

it("overlaps each pair of scenes by its own cut's length, and not at all at a plain cut", () => {
  fc.assert(
    fc.property(series, ([lengths, cutLengths]) => {
      const cuts = cutLengths.map((frames) => (frames === null ? null : cutOf(frames)));
      const overlaps = cutOverlaps(cuts, lengths.length);
      const starts = sceneStarts(lengths, overlaps);
      expect(starts[0]).toBe(0);
      expect(overlaps).toEqual(cutLengths.map((frames) => frames ?? 0));
      starts.slice(1).forEach((start, i) => {
        expect((starts[i] ?? 0) + (lengths[i] ?? 0) - start).toBe(overlaps[i]);
      });
      const end = (starts.at(-1) ?? 0) + (lengths.at(-1) ?? 0);
      const total = lengths.reduce((sum, length) => sum + length, 0);
      expect(end).toBe(total - overlaps.reduce((sum, frames) => sum + frames, 0));
    }),
  );
});

it("treats a missing cut as a plain cut", () => {
  fc.assert(
    fc.property(fc.integer({ min: 1, max: 10 }), (count) => {
      expect(cutOverlaps([], count)).toEqual(Array.from({ length: count - 1 }, () => 0));
    }),
  );
});
