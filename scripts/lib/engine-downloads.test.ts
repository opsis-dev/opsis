import { describe, expect, it } from "vitest";
import { browserVersion } from "./engine";
import { readEngineDownloads, SHELL_PLATFORMS, shellUrl } from "./engine-downloads";
import { cachedName } from "./pins";

describe("the pinned recording browser", () => {
  it("pins every platform at the address built today", () => {
    const downloads = readEngineDownloads();
    expect(downloads.shells.map((shell) => shell.platform)).toEqual([...SHELL_PLATFORMS]);
    for (const shell of downloads.shells)
      expect(shell.url).toBe(shellUrl(downloads.testedVersion, shell.platform));
  });

  it("reads the version from any Chromium build", () => {
    expect(browserVersion("Chromium 157.0.8080.0\n")).toBe("157.0.8080.0");
    expect(browserVersion("Google Chrome for Testing 157.0.8080.0 \n")).toBe("157.0.8080.0");
    expect(browserVersion("ffmpeg version 7.0.2")).toBeUndefined();
  });
});

describe("cached downloads", () => {
  it("keep the names the cache already holds", () => {
    expect(
      cachedName(
        "https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/ffmpeg-linux-x64.gz",
      ),
    ).toBe("7f0d765176f0e314-ffmpeg-linux-x64.gz");
    expect(cachedName("https://a.b/x/chrome.zip?clear")).toMatch(/^[0-9a-f]{16}-chrome\.zip$/);
  });
});
