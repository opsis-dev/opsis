import { spawn, type ChildProcess } from "node:child_process";
import path from "node:path";
import type { Readable, Writable } from "node:stream";
import { z } from "zod";
import { engineStatus, requireEngineFile } from "../../engine/paths";
import { CancelledError, onAbort, throwIfCancelled } from "../../shared/errors";
import { clamp01 } from "../../shared/math";

export type FfmpegTools = {
  run: (args: string[], opts?: RunOptions) => Promise<string>;
  pipe: (args: string[], opts?: RunOptions) => { stdin: Writable; done: Promise<string> };
  probe: (file: string) => Promise<ProbeResult>;
};

type RunOptions = {
  signal?: AbortSignal;
  durationMs?: number;
  onProgress?: (progress: number) => void;
  onLog?: (line: string) => void;
};

export type ProbeResult = {
  durationMs: number;
  format: string;
  video?: { codec: string; width: number; height: number; fps: number; pixFmt: string };
  audio?: { codec: string; sampleRate: number; channels: number };
  raw: unknown;
};

export type MediaDeps = {
  projectDir: string;
  resolve: (ref: string) => string;
  ffmpeg: FfmpegTools;
  signal?: AbortSignal;
};

export class ToolError extends Error {
  constructor(readonly output: string) {
    super("The video engine couldn't process a file.");
    this.name = "ToolError";
  }
}

const STDERR_LIMIT = 200_000;
const STDERR_KEEP = 100_000;
const STDERR_TAIL_LINES = 6;

type ToolProcess = ChildProcess & { stdout: Readable; stderr: Readable };

function progressReader({ onProgress, durationMs }: RunOptions): ((text: string) => void) | null {
  if (!onProgress) return null;
  return (text) => {
    if (!durationMs) return;
    for (const m of text.matchAll(/out_time_(?:us|ms)=(\d+)/g)) {
      const ms = Number(m[1]) / 1000;
      onProgress(clamp01(ms / durationMs));
    }
  };
}

function settle(child: ToolProcess, bin: string, opts: RunOptions): Promise<string> {
  return new Promise<string>((resolve, reject) => {
    let captured = "";
    let stderr = "";
    const stopListening = onAbort(opts.signal, () => child.kill("SIGKILL"));
    const readProgress = progressReader(opts);
    const capture = (text: string) => {
      captured += text;
    };
    const onStdout = readProgress ?? capture;

    child.stdout.on("data", (chunk: Buffer) => onStdout(chunk.toString()));
    child.stderr.on("data", (chunk: Buffer) => {
      const text = chunk.toString();
      stderr += text;
      if (stderr.length > STDERR_LIMIT) stderr = stderr.slice(-STDERR_KEEP);
      if (opts.onLog) for (const line of text.split(/\r?\n/)) if (line.trim()) opts.onLog(line);
    });
    child.on("error", (err) => {
      stopListening();
      reject(new ToolError(`${path.basename(bin)}: ${err.message}`));
    });
    child.on("close", (code) => {
      stopListening();
      if (opts.signal?.aborted) reject(new CancelledError());
      else if (code === 0) resolve(captured);
      else {
        const tail = stderr.split("\n").slice(-STDERR_TAIL_LINES).join("\n");
        reject(new ToolError(`${path.basename(bin)} exited with ${code}: ${tail}`));
      }
    });
  });
}

const spawnOptions = { windowsHide: true } as const;

export function createFfmpegTools(engineDir: string): FfmpegTools {
  const { ffmpeg, ffprobe } = engineStatus(engineDir);
  const ffmpegArgs = (args: string[], opts?: RunOptions) => [
    "-hide_banner",
    "-y",
    ...(opts?.onProgress ? ["-progress", "pipe:1", "-nostats"] : []),
    ...args,
  ];
  const run = async (bin: string | null, args: string[], opts: RunOptions = {}) => {
    const exe = requireEngineFile(bin);
    throwIfCancelled(opts.signal);
    const child = spawn(exe, args, { ...spawnOptions, stdio: ["ignore", "pipe", "pipe"] });
    return settle(child, exe, opts);
  };
  return {
    run: (args, opts) => run(ffmpeg, ffmpegArgs(args, opts), opts),
    pipe: (args, opts = {}) => {
      const exe = requireEngineFile(ffmpeg);
      throwIfCancelled(opts.signal);
      const child = spawn(exe, ffmpegArgs(args, opts), {
        ...spawnOptions,
        stdio: ["pipe", "pipe", "pipe"],
      });
      return { stdin: child.stdin, done: settle(child, exe, opts) };
    },
    probe: async (file) => {
      const out = await run(ffprobe, [
        "-v",
        "error",
        "-print_format",
        "json",
        "-show_format",
        "-show_streams",
        file,
      ]);
      return parseProbe(JSON.parse(out));
    },
  };
}

const rawStreamSchema = z.object({
  codec_type: z.string().optional(),
  codec_name: z.string().optional(),
  width: z.number().optional(),
  height: z.number().optional(),
  r_frame_rate: z.string().optional(),
  avg_frame_rate: z.string().optional(),
  pix_fmt: z.string().optional(),
  sample_rate: z.string().optional(),
  channels: z.number().optional(),
  duration: z.string().optional(),
});

const rawProbeSchema = z.object({
  format: z
    .object({ duration: z.string().optional(), format_name: z.string().optional() })
    .optional(),
  streams: z.array(rawStreamSchema).optional(),
});

export function parseProbe(json: unknown): ProbeResult {
  const parsed = rawProbeSchema.safeParse(json);
  if (!parsed.success) throw new ToolError(`ffprobe: unexpected output: ${parsed.error.message}`);
  const raw = parsed.data;
  const streams = raw.streams ?? [];
  const v = streams.find((s) => s.codec_type === "video");
  const a = streams.find((s) => s.codec_type === "audio");
  const rate = (r?: string) => {
    if (!r) return 0;
    const [n, d] = r.split("/").map(Number);
    return d ? n / d : n;
  };
  const duration = Number(raw.format?.duration ?? v?.duration ?? a?.duration ?? 0);
  return {
    durationMs: Number.isFinite(duration) ? Math.round(duration * 1000) : 0,
    format: raw.format?.format_name ?? "",
    video: v
      ? {
          codec: v.codec_name ?? "",
          width: v.width ?? 0,
          height: v.height ?? 0,
          fps: rate(v.avg_frame_rate) || rate(v.r_frame_rate),
          pixFmt: v.pix_fmt ?? "",
        }
      : undefined,
    audio: a
      ? {
          codec: a.codec_name ?? "",
          sampleRate: Number(a.sample_rate ?? 0),
          channels: a.channels ?? 0,
        }
      : undefined,
    raw: json,
  };
}

export function canStreamCopy(a: ProbeResult, b: ProbeResult): boolean {
  if (!a.video || !b.video) return false;
  return (
    a.video.codec === b.video.codec &&
    a.video.width === b.video.width &&
    a.video.height === b.video.height &&
    a.video.pixFmt === b.video.pixFmt &&
    Math.abs(a.video.fps - b.video.fps) < 0.01 &&
    (a.audio?.codec ?? "") === (b.audio?.codec ?? "") &&
    (a.audio?.sampleRate ?? 0) === (b.audio?.sampleRate ?? 0)
  );
}
