import { describe, expect, it } from "vitest";
import { namedFiles, namedProjectFiles } from "./named-files";

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
});
