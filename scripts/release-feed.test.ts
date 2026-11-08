import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { feed, releaseNotes, workspaceVersion } from "./release-feed";

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
              url: `https://github.com/bismitpanda/opsis/releases/download/v${shipped}/${file.file}`,
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

  it("reads the workspace version from the manifest", () => {
    fc.assert(
      fc.property(version, (shipped) => {
        const manifest = `[workspace]\nmembers = ["crates/*"]\n\n[workspace.package]\nversion = "${shipped}"\nedition = "2024"\n`;
        expect(workspaceVersion(manifest)).toBe(shipped);
      }),
    );
  });
});
