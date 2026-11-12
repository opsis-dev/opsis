import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { expect, it } from "vitest";
import { prepareMaskMorph } from "#video/elements/transition/transition-morph-shape/displacement-transition.ts";
import {
  MORPH_SHAPES,
  type MorphShapeName,
} from "#video/elements/transition/transition-morph-shape/path-morph.ts";

const FIXTURE = path.resolve(
  import.meta.dirname,
  "../../../../../../fixtures/scene/path-morph.json",
);

const SHAPES = Object.keys(MORPH_SHAPES) as MorphShapeName[];

function recorded() {
  return {
    pairs: SHAPES.flatMap((from) =>
      SHAPES.map((to) => {
        const pair = prepareMaskMorph({ from, to });
        return { from, to, start: pair.from, end: pair.to, closed: pair.closed };
      }),
    ),
  };
}

it("records the film's shape morphs, for the native renderer's port to match", () => {
  const made = recorded();
  if (process.env.OPSIS_WRITE_GOLDEN === "1") {
    mkdirSync(path.dirname(FIXTURE), { recursive: true });
    writeFileSync(FIXTURE, `${JSON.stringify(made, null, 1)}\n`);
  }
  expect(existsSync(FIXTURE), `${FIXTURE} is missing: record it with OPSIS_WRITE_GOLDEN=1`).toBe(
    true,
  );
  expect(JSON.parse(readFileSync(FIXTURE, "utf8"))).toEqual(JSON.parse(JSON.stringify(made)));
});
