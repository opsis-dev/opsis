import { describe, expect, it } from "vitest";
import { expandHome, joinFolder, projectFolderName } from "./project-folder";

describe("projectFolderName", () => {
  it("turns a project name into a lowercase folder name", () => {
    expect(projectFolderName("My launch video")).toBe("my-launch-video");
    expect(projectFolderName("  Q3 / Release!  ")).toBe("q3-release");
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

describe("expandHome", () => {
  it("replaces a leading ~ with the home folder", () => {
    expect(expandHome("~", "/home/me")).toBe("/home/me");
    expect(expandHome("~/Videos", "/home/me")).toBe("/home/me/Videos");
    expect(expandHome("~\\Videos", "C:\\Users\\me")).toBe("C:\\Users\\me\\Videos");
  });

  it("leaves other paths alone", () => {
    expect(expandHome("/srv/~demo", "/home/me")).toBe("/srv/~demo");
    expect(expandHome("~other/x", "/home/me")).toBe("~other/x");
  });
});
