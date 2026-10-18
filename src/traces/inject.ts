import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { strFromU8, strToU8, unzipSync, zipSync, type Zippable } from "fflate";
import { parseNdjson, traceTextPart } from "./model";

const TRACE_FRAME_FPS = 4;
const TRACE_FRAME_WIDTH = 960;
const TRACE_FRAME_JPEG_QUALITY = "5";
const STORED_UNCOMPRESSED = { level: 0 } as const;

type FfmpegRunner = { run: (args: string[]) => Promise<void> };

type RecordingFrame = { file: string; width: number; height: number; atMs: number };

const JPEG_MARK = 0xff;
const JPEG_START = 0xd8;
const MARKER_LENGTH = 2;
const FRAME_START_FIRST = 0xc0;
const FRAME_START_LAST = 0xc3;
const SEGMENT_LENGTH_AT = 2;
const FRAME_HEIGHT_AT = 5;
const FRAME_WIDTH_AT = 7;

export function jpegSize(data: Uint8Array): { width: number; height: number } | null {
  if (data[0] !== JPEG_MARK || data[1] !== JPEG_START) return null;
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let i = MARKER_LENGTH;
  while (i + FRAME_WIDTH_AT + 1 < data.length) {
    if (data[i] !== JPEG_MARK) return null;
    const marker = data[i + 1];
    if (marker === JPEG_MARK) {
      i++;
      continue;
    }
    if (marker >= FRAME_START_FIRST && marker <= FRAME_START_LAST)
      return {
        height: view.getUint16(i + FRAME_HEIGHT_AT),
        width: view.getUint16(i + FRAME_WIDTH_AT),
      };
    i += MARKER_LENGTH + view.getUint16(i + SEGMENT_LENGTH_AT);
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

type ExtractedFrame = RecordingFrame & { data: Uint8Array };

async function extractFrames(
  ffmpeg: FfmpegRunner,
  video: string,
  dir: string,
): Promise<ExtractedFrame[]> {
  await ffmpeg.run([
    "-i",
    video,
    "-vf",
    `fps=${TRACE_FRAME_FPS},scale=${TRACE_FRAME_WIDTH}:-2`,
    "-q:v",
    TRACE_FRAME_JPEG_QUALITY,
    path.join(dir, "f-%05d.jpg"),
  ]);
  const names = (await readdir(dir)).filter((n) => n.endsWith(".jpg")).sort();
  const frames: ExtractedFrame[] = [];
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

type TraceZip = { files: Record<string, Uint8Array>; traceName: string; text: string };

async function readTraceZip(zipFile: string): Promise<TraceZip | null> {
  const files = unzipSync(new Uint8Array(await readFile(zipFile)));
  const traceName = Object.keys(files).find((n) => traceTextPart(n) === "trace");
  return traceName ? { files, traceName, text: strFromU8(files[traceName]) } : null;
}

function appendFrames(zip: TraceZip, frames: ExtractedFrame[], lines: string[]): Uint8Array {
  const out: Zippable = { ...zip.files };
  for (const f of frames) out[f.file] = [f.data, STORED_UNCOMPRESSED];
  out[zip.traceName] = strToU8(`${zip.text.replace(/\n?$/, "\n")}${lines.join("\n")}\n`);
  return zipSync(out);
}

type FramesAdded = { added: true } | { added: false; reason: string };

const skipped = (reason: string): FramesAdded => ({ added: false, reason });

export async function addRecordingFrames(opts: {
  zipFile: string;
  video: string;
  originWallMs: number;
  workDir: string;
  ffmpeg: FfmpegRunner;
}): Promise<FramesAdded> {
  if (!existsSync(opts.zipFile) || !existsSync(opts.video))
    return skipped("the recording or its trace is missing");
  const dir = path.join(opts.workDir, "trace-frames");
  await rm(dir, { recursive: true, force: true });
  await mkdir(dir, { recursive: true });
  try {
    const frames = await extractFrames(opts.ffmpeg, opts.video, dir);
    if (!frames.length) return skipped("no frames came out of the recording");
    const zip = await readTraceZip(opts.zipFile);
    if (!zip) return skipped("the trace has no list of actions");
    const lines = frameEvents(zip.text, frames, opts.originWallMs);
    if (!lines.length) return skipped("the trace has no clock");
    await writeFile(opts.zipFile, appendFrames(zip, frames, lines));
    return { added: true };
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}
