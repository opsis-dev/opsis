import { describe, expect, it } from "vitest";
import { friendlyMixError } from "./render";

describe("mix errors", () => {
  it("reads the missing file from the tool's own wording", () => {
    const error = new Error(
      "ffmpeg exited with 254: [in#0 @ 0x1] Error opening input: No such file or directory\nError opening input file /home/me/project/assets/my song.mp3.\nError opening input files: No such file or directory",
    );
    expect(friendlyMixError(error).message).toBe(
      "The sound file “my song.mp3” is missing from the project.",
    );
  });

  it("names a missing sound file without naming the tools", () => {
    const error = new Error(
      "ffmpeg exited with 254: [in#0 @ 0x1] Error opening input: No such file or directory\n/home/me/project/assets/song.mp3: No such file or directory",
    );
    expect(friendlyMixError(error).message).toBe(
      "The sound file “song.mp3” is missing from the project.",
    );
    expect(friendlyMixError(new Error("ffmpeg exited with 1: bad")).message).not.toMatch(/ffmpeg/i);
  });
});
