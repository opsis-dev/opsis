import { readFileSync } from "node:fs";
import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { ffmpegReleaseUrl, readFfmpegPins } from "./lib/ffmpeg";
import { NOTICES } from "./lib/notices";
import { feed, githubNotes, releaseNotes, workspaceVersion } from "./release-feed";

const version = fc
  .tuple(fc.nat(40), fc.nat(40), fc.nat(40))
  .map(([major, minor, patch]) => `${major}.${minor}.${patch}`);
const line = fc.stringMatching(/^[A-Za-z][A-Za-z ,.]{0,30}$/);

describe("the release feed", () => {
  it("lists each update file under its own key at the release's download address", () => {
    fc.assert(
      fc.property(
        version,
        fc.uniqueArray(fc.stringMatching(/^[a-z]{2,8}-[a-z0-9_]{2,8}-[a-z]{2,8}$/), {
          maxLength: 6,
        }),
        (shipped, keys) => {
          const files = keys.map((key) => ({ key, file: `${key}.bin`, signature: `sig ${key}` }));
          const written = feed(shipped, "notes", files, "2026-11-08T00:00:00Z");
          expect(Object.keys(written.platforms).sort()).toEqual([...keys].sort());
          for (const file of files) {
            expect(written.platforms[file.key]).toEqual({
              url: `https://github.com/opsis-dev/opsis/releases/download/v${shipped}/${file.file}`,
              signature: file.signature,
            });
          }
        },
      ),
    );
  });

  it("refuses two update files for one system", () => {
    const twice = [
      { key: "linux-x86_64-appimage", file: "a", signature: "x" },
      { key: "linux-x86_64-appimage", file: "b", signature: "y" },
    ];
    expect(() => feed("1.0.0", "", twice, "")).toThrow("Two update files name the same system.");
  });

  it("takes the notes of the released version's section only", () => {
    fc.assert(
      fc.property(version, fc.array(line, { minLength: 1, maxLength: 5 }), (shipped, notes) => {
        const changelog = [
          "# Changelog",
          "",
          "## [Unreleased]",
          "",
          "- next",
          "",
          `## [${shipped}] - 2026-11-08`,
          "",
          ...notes,
          "",
          "## [0.0.0]",
          "",
          "- older",
        ].join("\n");
        expect(releaseNotes(changelog, shipped)).toBe(notes.join("\n").trim());
      }),
    );
  });

  it("follows the changelog notes with the FFmpeg notice and nothing else", () => {
    const body = fc.array(line, { minLength: 1, maxLength: 4 }).map((lines) => lines.join("\n"));
    fc.assert(
      fc.property(
        fc.array(line, { minLength: 1, maxLength: 5 }),
        body,
        fc.array(fc.tuple(fc.stringMatching(/^[A-EG-Z][a-z]{1,10}$/), body), { maxLength: 3 }),
        fc.array(fc.tuple(fc.stringMatching(/^[A-EG-Z][a-z]{1,10}$/), body), { maxLength: 3 }),
        (notes, ffmpegBody, before, after) => {
          const section = `## FFmpeg (opsis-ffmpeg)\n\n${ffmpegBody}`;
          const notices = [
            "# Third-party notices",
            ...before.map(([heading, text]) => `## ${heading}\n\n${text}`),
            section,
            ...after.map(([heading, text]) => `## ${heading}\n\n${text}`),
          ].join("\n\n");
          const changelog = notes.join("\n");
          expect(githubNotes(changelog, notices)).toBe(`${changelog}\n\n${section.trim()}\n`);
        },
      ),
    );
  });

  it("puts both FFmpeg source links of the shipped notice in the release notes", () => {
    const written = githubNotes("- notes", readFileSync(NOTICES, "utf8"));
    expect(written).toContain("https://ffmpeg.org/download.html");
    expect(written).toContain(ffmpegReleaseUrl(readFfmpegPins().release));
    expect(written).not.toContain("## Chrome Headless Shell");
  });

  it("reads the workspace version from the manifest", () => {
    fc.assert(
      fc.property(version, (shipped) => {
        const manifest = `[workspace]\nmembers = ["crates/*"]\n\n[workspace.package]\nversion = "${shipped}"\nedition = "2024"\n`;
        expect(workspaceVersion(manifest)).toBe(shipped);
      }),
    );
  });
});
