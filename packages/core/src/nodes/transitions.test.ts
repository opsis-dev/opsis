import { describe, expect, it } from "vitest";
import { emptyClip } from "#core/compile/clip";
import { ELEMENTS } from "#core/elements";
import { compileClip, nodeCard } from "#core/test-fixtures";

const leakIn = (a: number, b: number, length: number) => {
  const film = compileClip(
    "transition.lightLeak",
    { durationInFrames: length },
    { a: emptyClip(a), b: emptyClip(b) },
  );
  const leak = film.tracks[1].items[0];
  return { film: film.durationInFrames, from: leak.from, length: leak.durationInFrames };
};

describe("Light leak cut", () => {
  it("centres the leak on the cut", () => {
    expect(leakIn(90, 90, 30)).toEqual({ film: 180, from: 75, length: 30 });
  });

  it("keeps a long leak inside the film instead of making the film longer", () => {
    expect(leakIn(30, 30, 90)).toEqual({ film: 60, from: 0, length: 60 });
    expect(leakIn(10, 90, 40)).toEqual({ film: 100, from: 0, length: 40 });
  });

  it("warns when the leak is longer than both clips", () => {
    const texts = (a: number, b: number) =>
      nodeCard("transition.lightLeak", { durationInFrames: 90 }, { inputFrames: { a, b } }).map(
        (r) => r.text,
      );
    expect(texts(30, 30)).toEqual(["3 s", "Longer than both clips, so it plays for 2 s"]);
    expect(texts(60, 60)).toEqual(["3 s"]);
  });
});

describe("transitions with one clip", () => {
  const types = ELEMENTS.filter((meta) => meta.kind === "presentation").map(
    (meta) => `transition.${meta.key.replace("/", ".")}`,
  );

  it("covers every transition element", () => {
    expect(types.length).toBeGreaterThan(10);
  });

  it.each(types)("%s passes a lone clip through unchanged", (type) => {
    const first = emptyClip(40);
    const then = emptyClip(70);
    expect(compileClip(type, {}, { a: first })).toEqual(first);
    expect(compileClip(type, {}, { b: then })).toEqual(then);
  });
});
