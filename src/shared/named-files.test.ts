import { describe, expect, it } from "vitest";
import { namedFiles, namedProjectFiles, projectRefsIn } from "./named-files";

describe("namedProjectFiles", () => {
  it("finds the project files a node's settings name, nested or not", () => {
    const params = {
      file: "assets/looks/warm.cube",
      rows: [{ image: "project:assets/a.png" }],
      url: "https://example.com/page.html",
      sound: "app:sfx/click.wav",
      title: "Hello",
    };
    expect(namedFiles(params)).toHaveLength(4);
    expect(namedProjectFiles(params)).toEqual(["assets/looks/warm.cube", "project:assets/a.png"]);
  });

  it("names a dotLottie file", () => {
    expect(namedProjectFiles({ src: "assets/anim.lottie" })).toEqual(["assets/anim.lottie"]);
  });
});

describe("projectRefsIn", () => {
  it("finds every project ref once, whatever its extension, and nothing else", () => {
    const spec = {
      tracks: [{ items: [{ props: { src: "project:assets/anim.lottie" } }] }],
      audio: [{ src: "project:assets/tone.wav" }, { src: "project:assets/tone.wav" }],
      backdrop: "app:backdrops/aurora.jpg",
      title: "assets/not-a-ref.png",
    };
    expect(projectRefsIn(spec)).toEqual(["project:assets/anim.lottie", "project:assets/tone.wav"]);
  });
});
