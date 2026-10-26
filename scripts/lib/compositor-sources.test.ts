import { describe, expect, it } from "vitest";
import {
  appVersion,
  assetName,
  installedRemotion,
  readCompositorSources,
  sourcesNote,
} from "./compositor-sources";

describe("compositor sources", () => {
  const manifest = readCompositorSources();

  it("names the source of the Remotion version the app ships", () => {
    expect(manifest.remotion).toBe(installedRemotion());
  });

  it("gives every source its own asset name", () => {
    const names = manifest.sources.map((s) => assetName(s.url));
    expect(new Set(names).size).toBe(names.length);
  });

  it("reads the app's version from the Cargo workspace", () => {
    expect(appVersion()).toMatch(/^\d+\.\d+\.\d+/);
  });

  it("points the note at this release and every source", () => {
    const note = sourcesNote(manifest, "1.2.3");
    expect(note).toContain("/releases/tag/v1.2.3");
    for (const source of manifest.sources) expect(note).toContain(source.url);
  });
});
