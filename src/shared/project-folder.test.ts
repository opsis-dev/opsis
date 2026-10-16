import { describe, expect, it } from "vitest";
import { joinFolder, projectFolderName } from "./project-folder";

describe("projectFolderName", () => {
  it("turns a project name into a lowercase folder name", () => {
    expect(projectFolderName("My launch video")).toBe("my-launch-video");
    expect(projectFolderName("  Q3 / Release!  ")).toBe("q3-release-");
    expect(projectFolderName("v1.2_beta")).toBe("v1.2_beta");
  });

  it("falls back when nothing usable is left", () => {
    expect(projectFolderName("   ")).toBe("demo");
  });
});

describe("joinFolder", () => {
  it("joins with the platform separator and drops trailing separators", () => {
    expect(joinFolder("/home/me/DemoflowProjects/", "demo", "linux")).toBe(
      "/home/me/DemoflowProjects/demo",
    );
    expect(joinFolder("C:\\Users\\me\\DemoflowProjects", "demo", "win32")).toBe(
      "C:\\Users\\me\\DemoflowProjects\\demo",
    );
    expect(joinFolder("/", "demo", "darwin")).toBe("/demo");
  });
});
