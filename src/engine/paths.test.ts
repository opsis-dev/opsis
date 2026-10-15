import { describe, expect, it } from "vitest";
import { enginePaths, remotionPlatform } from "./paths";

describe("engine paths", () => {
  it("maps Node platforms to Remotion's download platforms", () => {
    expect(remotionPlatform("linux", "x64")).toBe("linux64");
    expect(remotionPlatform("linux", "arm64")).toBe("linux-arm64");
    expect(remotionPlatform("darwin", "arm64")).toBe("mac-arm64");
    expect(remotionPlatform("win32", "x64")).toBe("win64");
  });

  it("lays the shell out where Remotion's ensureBrowser puts it", () => {
    const p = enginePaths("/engine", "win32", "x64");
    expect(p.headlessShell.replace(/\\/g, "/")).toBe(
      "/engine/node_modules/.remotion/chrome-headless-shell/win64/chrome-headless-shell-win64/chrome-headless-shell.exe",
    );
    expect(p.ffprobe.replace(/\\/g, "/")).toBe("/engine/ffmpeg/ffprobe.exe");
  });
});
