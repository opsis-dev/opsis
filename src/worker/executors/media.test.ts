import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { defaultParams, getNodeDef } from "../../shared/nodes/registry";
import { must, workflowNode } from "../../shared/test-fixtures";
import type { ExecutorContext } from "../engine/context";
import { getExecutor } from "./index";
import { thumbnailTime } from "./media";

describe("thumbnailTime", () => {
  it("takes the frame at the chosen time when the file is long enough", () => {
    expect(thumbnailTime(1000, { durationMs: 4000 })).toBe(1000);
  });

  it("takes a frame just before the end of a shorter file", () => {
    expect(thumbnailTime(1000, { durationMs: 500 })).toBe(400);
    expect(thumbnailTime(60_000, { durationMs: 4000 })).toBe(3900);
  });

  it("takes the only frame of a still image", () => {
    expect(thumbnailTime(1000, { durationMs: 0 })).toBe(0);
  });
});

describe("Apply grade executor", () => {
  const applyGrade = must(getExecutor("media.applyGrade"), "Apply grade executor");
  const ran: string[][] = [];
  const video = { width: 1920, height: 1080, fps: 30, codec: "h264" };
  const probeOf = (sound: string | null) => ({
    durationMs: 4000,
    format: "mov,mp4",
    video,
    audio: sound ? { codec: sound, sampleRate: 48000 } : null,
    raw: {},
  });

  function contextFor(
    projectDir: string,
    params: Record<string, unknown>,
    options: { sound?: string | null; names?: Record<string, string> } = {},
  ) {
    const nodes = Object.entries(options.names ?? {}).map(([id, label]) => ({
      ...workflowNode("grade.lookFile", {}, id),
      label,
    }));
    return {
      projectDir,
      env: { isPackaged: false, appRoot: projectDir, resourcesPath: projectDir },
      dir: projectDir,
      relDir: ".opsis/cache/h",
      node: workflowNode("media.applyGrade", params, "apply"),
      workflow: { nodes },
      vars: {},
      signal: new AbortController().signal,
      log: () => {},
      progress: () => {},
      ffmpeg: {
        probe: async () => probeOf(options.sound === undefined ? "aac" : options.sound),
        run: async (args: string[]) => void ran.push(args),
      },
    } as unknown as ExecutorContext;
  }

  const media = (kind: "video" | "image" = "video") => ({
    type: "media" as const,
    kind,
    src: "project:assets/film.mp4",
  });

  const gradeOf = (...steps: Array<[string, Record<string, unknown>, string?]>) => ({
    type: "grade" as const,
    entries: steps.map(([type, params, nodeId = type]) => ({
      nodeId,
      type,
      params: must(getNodeDef(type), type).params.parse({ ...defaultParams(type), ...params }),
    })),
  });

  const argsOf = async (
    inputs: Parameters<typeof applyGrade.run>[1],
    params: Record<string, unknown> = {},
    options: Parameters<typeof contextFor>[2] = {},
  ) => {
    ran.length = 0;
    const dir = await mkdtemp(path.join(tmpdir(), "grade-test-"));
    try {
      await applyGrade.run(contextFor(dir, params, options), inputs).catch((error: unknown) => {
        if (!/probe|ENOENT|artifact/i.test(String(error))) throw error;
      });
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
    return ran[0];
  };

  it("asks for a grade", async () => {
    await expect(applyGrade.run(contextFor("/p", {}), { media: media() })).rejects.toThrow(
      "Connect a grade.",
    );
  });

  it("names the look file step that has no file", async () => {
    await expect(
      applyGrade.run(contextFor("/p", {}, { names: { look: "Film look" } }), {
        media: media(),
        grade: gradeOf(["grade.lookFile", {}, "look"]),
      }),
    ).rejects.toThrow("Choose a look file for “Film look”.");
  });

  it("names the look file step whose file is gone", async () => {
    await expect(
      applyGrade.run(contextFor("/p", {}, { names: { look: "Film look" } }), {
        media: media(),
        grade: gradeOf(["grade.lookFile", { file: "assets/looks/gone.cube" }, "look"]),
      }),
    ).rejects.toThrow("The look file for “Film look” isn't in the project.");
  });

  it("writes one pass: the grade graph, the picture format, and the sound copied", async () => {
    const args = await argsOf({
      media: media(),
      grade: gradeOf(["grade.colour", { contrast: 1.2 }]),
    });
    expect(args).toEqual([
      "-i",
      expect.stringMatching(/assets\/film\.mp4$/),
      "-filter_complex",
      "[0:v]eq=contrast=1.2,scale=trunc(iw/2)*2:trunc(ih/2)*2[out]",
      "-map",
      "[out]",
      "-map",
      "0:a?",
      "-c:v",
      "libx264",
      "-preset",
      "medium",
      "-crf",
      "18",
      "-pix_fmt",
      "yuv420p",
      "-c:a",
      "copy",
      "-movflags",
      "+faststart",
      expect.stringMatching(/graded\.mp4$/),
    ]);
  });

  it("maps no sound for a silent file, and writes a PNG for an image", async () => {
    const silent = await argsOf(
      { media: media(), grade: gradeOf(["grade.colour", { contrast: 1.2 }]) },
      {},
      { sound: null },
    );
    expect(silent).not.toContain("0:a?");
    const still = await argsOf({
      media: media("image"),
      grade: gradeOf(["grade.colour", { contrast: 1.2 }]),
    });
    expect(still).toEqual(
      expect.arrayContaining(["-frames:v", "1", expect.stringMatching(/graded\.png$/)]),
    );
    expect(still).not.toContain("-c:v");
  });
});
