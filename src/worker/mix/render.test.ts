import { describe, expect, it } from "vitest";
import { CancelledError } from "../../shared/errors";
import { ToolError } from "../media/ffmpeg";
import { friendlyMixError, parseEbur128, parseFrameLines, parseSummary, toFrames } from "./render";

describe("mix errors", () => {
  it.each([
    [
      "Error opening input: No such file or directory\nError opening input file /home/me/project/assets/my song.mp3.\nError opening input files: No such file or directory",
      "The sound file “my song.mp3” is missing from the project.",
    ],
    [
      "Error opening input: No such file or directory\n/home/me/project/assets/song.mp3: No such file or directory",
      "The sound file “song.mp3” is missing from the project.",
    ],
    ["bad", "The sound couldn't be mixed. Check that every sound file still opens."],
  ])("explains %j without naming the tools", (output, message) => {
    const error = new ToolError(`ffmpeg exited with 254: [in#0 @ 0x1] ${output}`);
    expect(friendlyMixError(error).message).toBe(message);
    expect(friendlyMixError(error).message).not.toMatch(/ffmpeg/i);
  });

  it("passes a cancel on, and only a real one", () => {
    const cancel = new CancelledError();
    expect(friendlyMixError(cancel)).toBe(cancel);
    expect(friendlyMixError(new Error("Cancelled"))).not.toBeInstanceOf(CancelledError);
  });
});

describe("parseEbur128", () => {
  const frame = (t: number, m: string, s: string) =>
    `[Parsed_ebur128_0 @ 0x1] t: ${t}  TARGET:-23 LUFS    M: ${m} S: ${s}     I: -20.0 LUFS       LRA:   0.0 LU`;
  const summary = [
    "[Parsed_ebur128_0 @ 0x1] Summary:",
    "  Integrated loudness:",
    "    I:         -16.04 LUFS",
    "    Threshold: -26.3 LUFS",
    "  Loudness range:",
    "    LRA:         4.26 LU",
    "    Threshold: -36.2 LUFS",
    "  True peak:",
    "    Peak:       -1.37 dBFS",
  ];

  it("reads the time, momentary and short-term readings of each frame line", () => {
    expect(
      parseFrameLines([frame(0.1, "-inf", "-inf"), "Summary:", frame(0.5, "-18.24", "-19.91")]),
    ).toEqual([
      { t: 0.1, m: -120, s: -120 },
      { t: 0.5, m: -18.24, s: -19.91 },
    ]);
  });

  it("reads each summary value under its own heading only", () => {
    expect(parseSummary([frame(0.1, "-20", "-20"), ...summary])).toEqual({
      integrated: -16.04,
      range: 4.26,
      truePeak: -1.37,
    });
    expect(parseSummary(["    I:         -16.04 LUFS", "  True peak:"])).toEqual({
      integrated: null,
      range: null,
      truePeak: null,
    });
  });

  it("holds the latest reading at the end of each frame and leaves earlier frames silent", () => {
    const samples = [
      { t: 0.6, m: -18.24, s: -19.91 },
      { t: 1.1, m: -10.04, s: -12.26 },
    ];
    expect(toFrames(samples, 2, 4)).toEqual({
      momentary: [-120, -18.2, -10, -10],
      shortTerm: [-120, -19.9, -12.3, -12.3],
    });
  });

  it("reads the summary and one reading per frame", () => {
    const lines = [frame(0.1, "-inf", "-inf"), frame(0.5, "-18.24", "-19.91"), ...summary];
    const loudness = parseEbur128(lines, 2, 3);
    expect(loudness).toEqual({
      fps: 2,
      momentary: [-18.2, -18.2, -18.2],
      shortTerm: [-19.9, -19.9, -19.9],
      integrated: -16,
      range: 4.3,
      truePeak: -1.4,
    });
  });

  it("reports silence as no reading", () => {
    const loudness = parseEbur128(
      [
        "  Integrated loudness:",
        "    I:         -70.0 LUFS",
        "  True peak:",
        "    Peak:   -inf dBFS",
      ],
      30,
      2,
    );
    expect(loudness.integrated).toBeNull();
    expect(loudness.truePeak).toBeNull();
    expect(loudness.range).toBeNull();
    expect(loudness.momentary).toEqual([-120, -120]);
  });
});
