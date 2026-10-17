import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { calmProgress, downloadedSize } from "./install";

describe("engine download progress", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("only reports when the shown percentage or the part changes, or every half second", () => {
    const seen: Array<[string, number | null]> = [];
    const calm = calmProgress(({ part, progress }) => seen.push([part, progress]));
    const report = (
      part: "ffmpeg" | "headless-shell",
      step: string,
      progress: number,
      message: string,
    ) => calm({ part, step, progress, message });
    report("ffmpeg", "download-1", 0.101, "Downloading the video tools (1 of 3) · 3 of 29 MB");
    report("ffmpeg", "download-1", 0.104, "Downloading the video tools (1 of 3) · 3 of 29 MB");
    report("ffmpeg", "download-1", 0.109, "Downloading the video tools (1 of 3) · 3 of 29 MB");
    vi.advanceTimersByTime(600);
    report("ffmpeg", "download-1", 0.109, "Downloading the video tools (1 of 3) · 3 of 29 MB");
    report("ffmpeg", "download-1", 0.12, "Downloading the video tools (1 of 3) · 4 of 29 MB");
    report("ffmpeg", "download-2", 0.12, "Downloading the video tools (2 of 3) · 4 of 29 MB");
    report("headless-shell", "download", 0.12, "Downloading the recording browser · 15 of 126 MB");
    expect(seen.map(([, p]) => p)).toEqual([0.101, 0.109, 0.12, 0.12, 0.12]);
    expect(seen.at(-1)?.[0]).toBe("headless-shell");
  });

  it("shows downloaded sizes", () => {
    expect(downloadedSize(12_400_000, 29_354_986)).toBe("11.8 MB of 28.0 MB");
    expect(downloadedSize(5_242_880, null)).toBe("5.0 MB");
    expect(downloadedSize(20_000, 35_147)).toBe("19.5 KB of 34.3 KB");
  });
});
