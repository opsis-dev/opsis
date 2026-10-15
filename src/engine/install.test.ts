import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { calmProgress, megabytes } from "./install";

describe("engine download progress", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("only reports when the shown percentage or the part changes, or every half second", () => {
    const seen: Array<[string, number | null]> = [];
    const report = calmProgress((part, progress) => seen.push([part, progress]));
    report("ffmpeg", 0.101, "Downloading the video tools (1 of 3) · 3 of 29 MB");
    report("ffmpeg", 0.104, "Downloading the video tools (1 of 3) · 3 of 29 MB");
    report("ffmpeg", 0.109, "Downloading the video tools (1 of 3) · 3 of 29 MB");
    vi.advanceTimersByTime(600);
    report("ffmpeg", 0.109, "Downloading the video tools (1 of 3) · 3 of 29 MB");
    report("ffmpeg", 0.12, "Downloading the video tools (1 of 3) · 4 of 29 MB");
    report("ffmpeg", 0.12, "Downloading the video tools (2 of 3) · 4 of 29 MB");
    report("headless-shell", 0.12, "Downloading the recording browser · 15 of 126 MB");
    expect(seen.map(([, p]) => p)).toEqual([0.101, 0.109, 0.12, 0.12, 0.12]);
    expect(seen.at(-1)?.[0]).toBe("headless-shell");
  });

  it("shows sizes in megabytes", () => {
    expect(megabytes(12_400_000, 29_354_986)).toBe("12 of 29 MB");
    expect(megabytes(5_000_000, null)).toBe("5 MB");
    expect(megabytes(20_000, 35_147)).toBe("36 KB");
  });
});
