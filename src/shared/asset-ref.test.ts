import { describe, expect, it } from "vitest";
import { baseName, encodePath, projectRelPath } from "./asset-ref";

describe("asset refs", () => {
  it("encodes each path segment", () => {
    expect(encodePath("assets/my clip#1.mp4")).toBe("assets/my%20clip%231.mp4");
  });

  it("strips the project prefix and leading slashes", () => {
    expect(projectRelPath("project:/assets/a.mp4")).toBe("assets/a.mp4");
    expect(projectRelPath("assets/a.mp4")).toBe("assets/a.mp4");
  });

  it("names the last segment, ignoring a trailing slash", () => {
    expect(baseName("project:assets/a.mp4")).toBe("a.mp4");
    expect(baseName("exports/site/")).toBe("site");
    expect(baseName("clip.mp4")).toBe("clip.mp4");
  });
});
