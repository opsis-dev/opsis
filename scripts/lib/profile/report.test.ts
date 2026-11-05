import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { overlapShare, type Sample } from "./report";

function sample(from: number, at: number): Sample {
  return { from, at, cpu: {}, memory: {} };
}

describe("splitting a sample between runs", () => {
  it("gives a sample's whole time out across runs that tile the time line", () => {
    fc.assert(
      fc.property(
        fc.nat({ max: 10_000 }),
        fc.integer({ min: 1, max: 5000 }),
        fc.array(fc.integer({ min: 1, max: 3000 }), { minLength: 1, maxLength: 12 }),
        (from, length, gaps) => {
          const edges = [from - 1];
          for (const gap of gaps) edges.push(edges.at(-1)! + gap);
          edges.push(from + length + 1);
          const runs = edges.slice(1).map((end, index) => [edges[index], end]);
          const shares = runs.map(([start, end]) =>
            overlapShare(sample(from, from + length), start, end),
          );
          expect(shares.reduce((sum, share) => sum + share, 0)).toBeCloseTo(1, 9);
          expect(shares.every((share) => share >= 0 && share <= 1)).toBe(true);
        },
      ),
    );
  });
});
