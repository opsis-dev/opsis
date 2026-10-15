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
});
