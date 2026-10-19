import { render } from "@testing-library/react";
import { isValidElement } from "react";
import { describe, expect, it } from "vitest";
import type { ElementProps } from "../../shared/elements/types";
import {
  listsFromRows,
  marksFromImages,
  nested,
  themed,
  ticksFromWords,
  treeFrom,
  treeFromPaths,
} from "./framing";
import type { ElementComponent } from "./types";

function spy() {
  const seen: ElementProps[] = [];
  const Component: ElementComponent = (props) => {
    seen.push(props);
    return null;
  };
  return { Component, last: () => seen.at(-1) ?? {} };
}

describe("nested", () => {
  it("builds nested props from flat settings and leaves unset ones out", () => {
    const { Component, last } = spy();
    const Nested = nested(Component, {
      runTitle: "run.title",
      runPlan: "run.plan",
      bg: "theme.background",
    });
    render(<Nested runTitle="Hello" runPlan={["a"]} bg={undefined} other={1} />);
    expect(last()).toEqual({ other: 1, run: { title: "Hello", plan: ["a"] } });
  });
});

describe("listsFromRows", () => {
  it("splits rows into one list per column", () => {
    const { Component, last } = spy();
    const Lists = listsFromRows(Component, { bars: { label: "labels", value: "data" } });
    render(
      <Lists
        bars={[
          { label: "Jan", value: 3 },
          { label: "Feb", value: 5 },
        ]}
      />,
    );
    expect(last()).toEqual({ labels: ["Jan", "Feb"], data: [3, 5] });
  });

  it("leaves out a column that is empty in every row", () => {
    const { Component, last } = spy();
    const Lists = listsFromRows(Component, { bars: { label: "labels", value: "data" } });
    render(<Lists bars={[{ label: "", value: 3 }]} />);
    expect(last()).toEqual({ data: [3] });
  });
});

describe("marksFromImages", () => {
  it("turns image rows into marks with their backgrounds", () => {
    const { Component, last } = spy();
    const Marks = marksFromImages(Component, { from: "logos", to: "logos" });
    render(<Marks logos={[{ image: "/a.svg", background: "#000" }, { image: "" }]} />);
    const logos = last().logos;
    expect(Array.isArray(logos) && logos.length).toBe(2);
    const [first, second] = Array.isArray(logos) ? logos : [];
    expect(first.bg).toBe("#000");
    expect(isValidElement(first.mark) && first.mark.props).toMatchObject({ src: "/a.svg" });
    expect(second).toEqual({ mark: null });
  });
});

describe("ticksFromWords", () => {
  it("turns yes and no cells into ticks and dashes", () => {
    const { Component, last } = spy();
    const Ticks = ticksFromWords(Component, "rows");
    render(<Ticks rows={[{ label: "4K", cells: ["Yes", "no", "60"] }]} />);
    expect(last().rows).toEqual([{ label: "4K", cells: [true, false, "60"] }]);
  });
});

describe("treeFrom", () => {
  it("builds folders from paths", () => {
    expect(treeFromPaths(["src/a.ts", "src/lib/b.ts", "docs/", "README.md"])).toEqual([
      { name: "src", children: [{ name: "a.ts" }, { name: "lib", children: [{ name: "b.ts" }] }] },
      { name: "docs", children: [] },
      { name: "README.md" },
    ]);
    const { Component, last } = spy();
    const Tree = treeFrom(Component, "nodes");
    render(<Tree nodes={["a/b.ts"]} />);
    expect(last().nodes).toEqual([{ name: "a", children: [{ name: "b.ts" }] }]);
  });
});

describe("themed", () => {
  it("passes the mode and the changed colours", () => {
    const { Component, last } = spy();
    const Themed = themed(Component, "prop");
    render(<Themed theme={{ mode: "light", colors: { background: "#fff", card: " " } }} />);
    expect(last()).toEqual({ theme: { background: "#fff" }, mode: "light" });
  });

  it("gives remocn elements the whole preset with the changed colours on top", () => {
    const { Component, last } = spy();
    const Themed = themed(Component, "context");
    render(<Themed theme={{ mode: "dark", colors: { background: "#123456" } }} />);
    expect(last().theme).toMatchObject({ background: "#123456", foreground: "oklch(0.985 0 0)" });
  });

  it("leaves props alone without a theme setting", () => {
    const { Component, last } = spy();
    const Themed = themed(Component, "prop");
    render(<Themed mode="dark" />);
    expect(last()).toEqual({ mode: "dark" });
  });
});
