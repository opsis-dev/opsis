import { describe, expect, it } from "vitest";
import type { ProbeResult } from "../media/ffmpeg";
import { mediaHasSound } from "./util";

const probed = (audio: boolean) => ({
  projectDir: "/project",
  env: { isPackaged: false, appRoot: "/app", resourcesPath: "/resources" },
  ffmpeg: {
    probe: (): Promise<ProbeResult> =>
      Promise.resolve({
        format: "mp4",
        durationMs: 1000,
        raw: null,
        ...(audio ? { audio: { codec: "aac", sampleRate: 48_000, channels: 2 } } : {}),
      }),
  },
});

const film = { type: "media", src: "project:out/film.mp4", kind: "video" } as const;

describe("mediaHasSound", () => {
  it("probes the file when the value doesn't say", async () => {
    expect(await mediaHasSound(probed(true), film)).toBe(true);
    expect(await mediaHasSound(probed(false), film)).toBe(false);
  });

  it("trusts the value when it says", async () => {
    expect(await mediaHasSound(probed(true), { ...film, hasAudio: false })).toBe(false);
  });
});
