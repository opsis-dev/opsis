import { describe, expect, it } from "vitest";
import {
  baseName,
  extensionOf,
  fileVersionKey,
  joinUrl,
  projectRef,
  projectRelPath,
  refTarget,
  versionedUrl,
} from "./asset-ref";

describe("asset refs", () => {
  it("joins a base and an encoded path, with or without the base's slash", () => {
    expect(joinUrl("opsis-file://project/", "assets/my clip#1.mp4")).toBe(
      "opsis-file://project/assets/my%20clip%231.mp4",
    );
    expect(joinUrl("http://127.0.0.1:5000", "a b.png")).toBe("http://127.0.0.1:5000/a%20b.png");
  });

  it("strips the project prefix and leading slashes", () => {
    expect(projectRelPath("project:/assets/a.mp4")).toBe("assets/a.mp4");
    expect(projectRelPath("assets/a.mp4")).toBe("assets/a.mp4");
    expect(projectRelPath("app:sfx/a.wav")).toBeNull();
  });

  it("sorts every ref into project, app, web and absolute files", () => {
    expect(refTarget("project:assets/a.png")).toEqual({ kind: "project", rel: "assets/a.png" });
    expect(refTarget("assets/a.png")).toEqual({ kind: "project", rel: "assets/a.png" });
    expect(refTarget("app:backdrops/a.jpg")).toEqual({ kind: "app", rel: "backdrops/a.jpg" });
    expect(refTarget("https://x.test/a.png")).toEqual({
      kind: "external",
      url: "https://x.test/a.png",
    });
    expect(refTarget("/abs/a.png")).toEqual({ kind: "absolute", path: "/abs/a.png" });
    expect(refTarget("C:\\clips\\a.mp4")).toEqual({ kind: "absolute", path: "C:\\clips\\a.mp4" });
  });

  it("makes project refs of relative paths only", () => {
    expect(projectRef("assets/a.png")).toBe("project:assets/a.png");
    expect(projectRef("project:assets/a.png")).toBe("project:assets/a.png");
    expect(projectRef("/abs/a.png")).toBe("/abs/a.png");
    expect(projectRef("app:sfx/a.wav")).toBe("app:sfx/a.wav");
  });

  it("names the last segment, ignoring a trailing slash", () => {
    expect(baseName("project:assets/a.mp4")).toBe("a.mp4");
    expect(baseName("exports/site/")).toBe("site");
    expect(baseName("clip.mp4")).toBe("clip.mp4");
  });
});

describe("extensionOf", () => {
  it("reads the last segment's extension in lowercase", () => {
    expect(extensionOf("assets/Clip.MP4")).toBe("mp4");
    expect(extensionOf("README")).toBe("");
    expect(extensionOf("dir.v2/clip")).toBe("");
    expect(extensionOf(".gitignore")).toBe("");
  });
});

describe("file versions", () => {
  it("keys a project file the same however its path is written", () => {
    expect(fileVersionKey("project:assets/./a.png")).toBe("assets/a.png");
    expect(fileVersionKey("project:/assets//a.png")).toBe("assets/a.png");
    expect(fileVersionKey("assets\\a.png")).toBe("assets/a.png");
    expect(fileVersionKey("app:backdrops/a.png")).toBeNull();
  });

  it("adds a changed file's version to its address", () => {
    const versions = { "assets/a.png": "12:3.5" };
    expect(
      versionedUrl("opsis-file://project/assets/a.png", "project:assets/a.png", versions),
    ).toBe("opsis-file://project/assets/a.png?v=12%3A3.5");
    expect(versionedUrl("opsis-file://project/b.png", "project:b.png", versions)).toBe(
      "opsis-file://project/b.png",
    );
    expect(versionedUrl("opsis-file://project/assets/a.png", "project:assets/a.png")).toBe(
      "opsis-file://project/assets/a.png",
    );
  });
});
