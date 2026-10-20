import { describe, expect, it } from "vitest";
import { nodeHash } from "./cache";

describe("nodeHash", () => {
  it("is stable under key order and changes with params or upstream", () => {
    const a = nodeHash("fx.core.title", { title: "Hi", fontSize: 100 }, [["clip", "x:clip"]]);
    const b = nodeHash("fx.core.title", { fontSize: 100, title: "Hi" }, [["clip", "x:clip"]]);
    expect(a).toBe(b);
    expect(
      nodeHash("fx.core.title", { title: "Hello", fontSize: 100 }, [["clip", "x:clip"]]),
    ).not.toBe(a);
    expect(
      nodeHash("fx.core.title", { title: "Hi", fontSize: 100 }, [["clip", "y:clip"]]),
    ).not.toBe(a);
  });

  it("doesn't depend on the order the inputs are listed in", () => {
    const ports: Array<[string, string]> = [
      ["clip", "x:clip"],
      ["audio", "y:audio"],
    ];
    expect(nodeHash("fx.core.title", {}, ports)).toBe(
      nodeHash("fx.core.title", {}, ports.toReversed()),
    );
    expect(nodeHash("fx.core.title", {}, ports)).not.toBe(
      nodeHash("fx.core.title", {}, [
        ["clip", "y:audio"],
        ["audio", "x:clip"],
      ]),
    );
  });
});

describe("nodeHash for a port that takes several connections", () => {
  it("doesn't depend on the order the connections were made", () => {
    const one = nodeHash("audio.mix", {}, [
      ["sounds", "a:audio"],
      ["sounds", "b:audio"],
    ]);
    const two = nodeHash("audio.mix", {}, [
      ["sounds", "b:audio"],
      ["sounds", "a:audio"],
    ]);
    expect(one).toBe(two);
  });
});

describe("nodeHash across releases", () => {
  it("changes with the app's version, so a new release never reads old results", () => {
    expect(nodeHash("media.trim", {}, [], "0.1.0")).not.toBe(
      nodeHash("media.trim", {}, [], "0.2.0"),
    );
  });
});
