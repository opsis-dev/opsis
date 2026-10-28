import { describe, expect, it } from "vitest";
import { listsFrom, nestedFrom, ticksFromWords, treeFromPaths } from "#video/lib/settings/shape.ts";

describe("nestedFrom", () => {
  it("builds nested props from flat settings and leaves unset ones out", () => {
    const paths = { runTitle: "run.title", runPlan: "run.plan", bg: "theme.background" };
    expect(
      nestedFrom({ runTitle: "Hello", runPlan: ["a"], bg: undefined, other: 1 }, paths),
    ).toEqual({
      other: 1,
      run: { title: "Hello", plan: ["a"] },
    });
  });
});

describe("listsFrom", () => {
  const lists = { bars: { label: "labels", value: "data" } };

  it("splits rows into one list per column", () => {
    const bars = [
      { label: "Jan", value: 3 },
      { label: "Feb", value: 5 },
    ];
    expect(listsFrom({ bars }, lists)).toEqual({ labels: ["Jan", "Feb"], data: [3, 5] });
  });

  it("leaves out a column that is empty in every row", () => {
    expect(listsFrom({ bars: [{ label: "", value: 3 }] }, lists)).toEqual({ data: [3] });
  });
});

describe("ticksFromWords", () => {
  it("turns yes and no cells into ticks and dashes", () => {
    expect(ticksFromWords([{ label: "4K", cells: ["Yes", "no", "60"] }])).toEqual([
      { label: "4K", cells: [true, false, "60"] },
    ]);
  });
});

describe("treeFromPaths", () => {
  it("builds folders from paths", () => {
    expect(treeFromPaths(["src/a.ts", "src/lib/b.ts", "docs/", "README.md"])).toEqual([
      { name: "src", children: [{ name: "a.ts" }, { name: "lib", children: [{ name: "b.ts" }] }] },
      { name: "docs", children: [] },
      { name: "README.md" },
    ]);
  });
});
