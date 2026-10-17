import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { strFromU8, strToU8, unzipSync, zipSync, type Zippable } from "fflate";
import { parseNdjson } from "./model";

const TRACE_FRAME_FPS = 4;
const TRACE_FRAME_WIDTH = 960;

type FfmpegRunner = { run: (args: string[]) => Promise<string> };

type RecordingFrame = { file: string; width: number; height: number; atMs: number };

export function jpegSize(data: Uint8Array): { width: number; height: number } | null {
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let i = 2;
  while (i + 8 < data.length) {
    if (data[i] !== 0xff) return null;
    const marker = data[i + 1];
    if (marker >= 0xc0 && marker <= 0xc3)
      return { height: view.getUint16(i + 5), width: view.getUint16(i + 7) };
    i += 2 + view.getUint16(i + 2);
  }
  return null;
}

export function frameEvents(
  traceText: string,
  frames: RecordingFrame[],
  originWallMs: number,
): string[] {
  const options = parseNdjson(traceText).find((e) => e.type === "context-options");
  const wallTime = options?.wallTime;
  const monotonicTime = options?.monotonicTime;
  if (typeof wallTime !== "number" || typeof monotonicTime !== "number") return [];
  return frames.map((f) =>
    JSON.stringify({
      type: "screencast-frame",
      pageId: "demoflow-recording",
      file: f.file,
      width: f.width,
      height: f.height,
      timestamp: monotonicTime + (originWallMs + f.atMs - wallTime),
    }),
  );
}

async function extractFrames(
  ffmpeg: FfmpegRunner,
  video: string,
  dir: string,
): Promise<Array<RecordingFrame & { data: Uint8Array }>> {
  await ffmpeg.run([
    "-i",
    video,
    "-vf",
    `fps=${TRACE_FRAME_FPS},scale=${TRACE_FRAME_WIDTH}:-2`,
    "-q:v",
    "5",
    path.join(dir, "f-%05d.jpg"),
  ]);
  const names = (await readdir(dir)).filter((n) => n.endsWith(".jpg")).sort();
  const frames: Array<RecordingFrame & { data: Uint8Array }> = [];
  for (const [k, name] of names.entries()) {
    const data = new Uint8Array(await readFile(path.join(dir, name)));
    const size = jpegSize(data);
    if (!size) continue;
    frames.push({
      file: `screencast/demoflow-${String(k).padStart(5, "0")}.jpeg`,
      data,
      ...size,
      atMs: (k * 1000) / TRACE_FRAME_FPS,
    });
  }
  return frames;
}

export async function addRecordingFrames(opts: {
  zipFile: string;
  video: string;
  originWallMs: number;
  workDir: string;
  ffmpeg: FfmpegRunner;
}): Promise<void> {
  if (!existsSync(opts.zipFile) || !existsSync(opts.video)) return;
  const dir = path.join(opts.workDir, "trace-frames");
  await rm(dir, { recursive: true, force: true });
  await mkdir(dir, { recursive: true });
  try {
    const frames = await extractFrames(opts.ffmpeg, opts.video, dir);
    if (!frames.length) return;
    const files = unzipSync(new Uint8Array(await readFile(opts.zipFile)));
    const traceName = Object.keys(files).find((n) => /(^|\/)(\d+-)?trace\.trace$/.test(n));
    if (!traceName) return;
    const text = strFromU8(files[traceName]);
    const lines = frameEvents(text, frames, opts.originWallMs);
    if (!lines.length) return;
    const out: Zippable = { ...files };
    for (const f of frames) out[f.file] = [f.data, { level: 0 }];
    out[traceName] = strToU8(`${text.replace(/\n?$/, "\n")}${lines.join("\n")}\n`);
    await writeFile(opts.zipFile, zipSync(out));
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}
