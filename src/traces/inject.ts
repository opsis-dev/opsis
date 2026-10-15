import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { strFromU8, strToU8, unzipSync, zipSync, type Zippable } from "fflate";

export const TRACE_FRAME_FPS = 4;
export const TRACE_FRAME_WIDTH = 960;

type Runner = { run: (args: string[]) => Promise<string> };

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
  frames: Array<{ file: string; width: number; height: number; atMs: number }>,
  originWallMs: number,
): string[] {
  const options = traceText
    .split("\n")
    .map((line) => {
      try {
        return JSON.parse(line) as { type?: string; wallTime?: number; monotonicTime?: number };
      } catch {
        return null;
      }
    })
    .find((e) => e?.type === "context-options");
  if (options?.wallTime === undefined || options.monotonicTime === undefined) return [];
  const toMono = (wall: number) => options.monotonicTime! + (wall - options.wallTime!);
  return frames.map((f) =>
    JSON.stringify({
      type: "screencast-frame",
      pageId: "demoflow-recording",
      file: f.file,
      width: f.width,
      height: f.height,
      timestamp: toMono(originWallMs + f.atMs),
    }),
  );
}

export async function addRecordingFrames(opts: {
  zipFile: string;
  video: string;
  originWallMs: number;
  workDir: string;
  ffmpeg: Runner;
}): Promise<number> {
  if (!existsSync(opts.zipFile) || !existsSync(opts.video)) return 0;
  const dir = path.join(opts.workDir, "trace-frames");
  await rm(dir, { recursive: true, force: true });
  await mkdir(dir, { recursive: true });
  try {
    await opts.ffmpeg.run([
      "-i",
      opts.video,
      "-vf",
      `fps=${TRACE_FRAME_FPS},scale=${TRACE_FRAME_WIDTH}:-2`,
      "-q:v",
      "5",
      path.join(dir, "f-%05d.jpg"),
    ]);
    const names = (await readdir(dir)).filter((n) => n.endsWith(".jpg")).sort();
    if (!names.length) return 0;
    const files = unzipSync(new Uint8Array(await readFile(opts.zipFile)));
    const traceName = Object.keys(files).find((n) => /(^|\/)(\d+-)?trace\.trace$/.test(n));
    if (!traceName) return 0;
    const added: Array<{ file: string; width: number; height: number; atMs: number }> = [];
    const out: Zippable = {};
    for (const [name, data] of Object.entries(files)) out[name] = data;
    for (const [k, name] of names.entries()) {
      const data = new Uint8Array(await readFile(path.join(dir, name)));
      const size = jpegSize(data);
      if (!size) continue;
      const file = `screencast/demoflow-${String(k).padStart(5, "0")}.jpeg`;
      out[file] = [data, { level: 0 }];
      added.push({ file, ...size, atMs: (k * 1000) / TRACE_FRAME_FPS });
    }
    const text = strFromU8(files[traceName]);
    const lines = frameEvents(text, added, opts.originWallMs);
    if (!lines.length) return 0;
    out[traceName] = strToU8(`${text.replace(/\n?$/, "\n")}${lines.join("\n")}\n`);
    await writeFile(opts.zipFile, zipSync(out));
    return lines.length;
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}
