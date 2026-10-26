import { describe, expect, it } from "vitest";
import { projectRefsIn } from "./named-files";

describe("files the editor loads", () => {
  it("watches a prepared file's project source, once, and never an app file", () => {
    expect(
      projectRefsIn({
        a: "project:assets/clip.mp4",
        b: ["prepared:model/project/assets/robot.glb", "project:assets/clip.mp4"],
        c: "prepared:typeface/700/app/fonts/bundled/inter/inter-latin-wght-normal.woff2",
        d: "app:sfx/click.mp3",
      }),
    ).toEqual(["project:assets/clip.mp4", "project:assets/robot.glb"]);
  });
});
