import { describe, expect, it } from "vitest";
import { splitMark } from "./title";

describe("marked words in a title", () => {
  it("finds the words whatever their case and keeps the title's own spelling", () => {
    expect(splitMark("Sign up in 10 seconds", "SECONDS")).toEqual([
      "Sign up in 10 ",
      "seconds",
      "",
    ]);
    expect(splitMark("Ship faster today", " faster ")).toEqual(["Ship ", "faster", " today"]);
  });

  it("marks nothing when the words are empty or not in the title", () => {
    expect(splitMark("Ship faster", "")).toBeNull();
    expect(splitMark("Ship faster", "slower")).toBeNull();
  });
});
