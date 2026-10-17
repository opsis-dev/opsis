import { spawn, type ChildProcess } from "node:child_process";
import path from "node:path";
import type { Readable, Writable } from "node:stream";
import { engineStatus } from "../../engine/paths";
import { ENGINE_MISSING_MESSAGE } from "../../shared/engine";
import { clamp } from "../../shared/math";

export type FfmpegTools = {
  run: (args: string[], opts?: RunOptions) => Promise<string>;
  pipe: (args: string[], opts?: RunOptions) => { stdin: Writable; done: Promise<string> };
  probe: (file: string) => Promise<ProbeResult>;
};

export type RunOptions = {
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

function cancelled(signal?: AbortSignal) {
  if (signal?.aborted) throw new Error("Cancelled");
}

function settle(child: ToolProcess, bin: string, opts: RunOptions): Promise<string> {
  return new Promise<string>((resolve, reject) => {
    let stdout = "";
    let stderr = "";
    const onAbort = () => child.kill("SIGKILL");
    opts.signal?.addEventListener("abort", onAbort, { once: true });

    child.stdout.on("data", (chunk: Buffer) => {
      const text = chunk.toString();
      stdout += text;
      if (opts.onProgress && opts.durationMs) {
        for (const m of text.matchAll(/out_time_(?:us|ms)=(\d+)/g)) {
          const ms = Number(m[1]) / 1000;
          opts.onProgress(clamp(ms / opts.durationMs, 0, 1));
        }
      }
    });
    child.stderr.on("data", (chunk: Buffer) => {
      const text = chunk.toString();
      stderr += text;
      if (stderr.length > STDERR_LIMIT) stderr = stderr.slice(-STDERR_KEEP);
      if (opts.onLog) for (const line of text.split(/\r?\n/)) if (line.trim()) opts.onLog(line);
    });
    child.on("error", (err) => {
      opts.signal?.removeEventListener("abort", onAbort);
      reject(new ToolError(`${path.basename(bin)}: ${err.message}`));
    });
    child.on("close", (code) => {
      opts.signal?.removeEventListener("abort", onAbort);
      if (opts.signal?.aborted) reject(new Error("Cancelled"));
      else if (code === 0) resolve(stdout);
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
  const need = (bin: string | null) => {
    if (!bin) throw new Error(ENGINE_MISSING_MESSAGE);
    return bin;
  };
  const ffmpegArgs = (args: string[], opts?: RunOptions) => [
    "-hide_banner",
    "-y",
    ...(opts?.onProgress ? ["-progress", "pipe:1", "-nostats"] : []),
    ...args,
  ];
  const run = (bin: string, args: string[], opts: RunOptions = {}): Promise<string> => {
    cancelled(opts.signal);
    const child = spawn(bin, args, { ...spawnOptions, stdio: ["ignore", "pipe", "pipe"] });
    return settle(child, bin, opts);
  };
  return {
    run: async (args, opts) => run(need(ffmpeg), ffmpegArgs(args, opts), opts),
    pipe: (args, opts = {}) => {
      const bin = need(ffmpeg);
      cancelled(opts.signal);
      const child = spawn(bin, ffmpegArgs(args, opts), {
        ...spawnOptions,
        stdio: ["pipe", "pipe", "pipe"],
      });
      return { stdin: child.stdin, done: settle(child, bin, opts) };
    },
    probe: async (file) => {
      const out = await run(need(ffprobe), [
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

type RawStream = {
  codec_type?: string;
  codec_name?: string;
  width?: number;
  height?: number;
  r_frame_rate?: string;
  avg_frame_rate?: string;
  pix_fmt?: string;
  sample_rate?: string;
  channels?: number;
  duration?: string;
};

export function parseProbe(raw: {
  format?: { duration?: string; format_name?: string };
  streams?: RawStream[];
}): ProbeResult {
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
    raw,
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
