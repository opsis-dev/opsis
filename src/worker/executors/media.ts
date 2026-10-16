import { writeFile } from "node:fs/promises";
import path from "node:path";
import { getNodeDef } from "../../shared/nodes/registry";
import type { CompileValue, MediaValue } from "../../shared/schema/values";
import type { Executor, ExecutorContext, ExecutorOutput } from "../engine/context";
import { canStreamCopy } from "../media/ffmpeg";
import { refToPath, rel, requireMedia } from "./util";

function mediaExec<P>(
  type: string,
  run: (
    ctx: ExecutorContext,
    inputs: Partial<Record<string, CompileValue>>,
    params: P,
  ) => Promise<ExecutorOutput>,
): Executor<P> {
  if (!getNodeDef(type)) throw new Error(`Missing node definition ${type}`);
  return { type, run };
}

async function mediaResult(
  ctx: ExecutorContext,
  file: string,
  label: string,
  kind?: MediaValue["kind"],
): Promise<ExecutorOutput> {
  const probe = await ctx.ffmpeg.probe(path.join(ctx.dir, file));
  const media: MediaValue = {
    type: "media",
    src: ctx.ref(file),
    kind: kind ?? (probe.video ? (probe.durationMs > 0 ? "video" : "image") : "audio"),
    durationMs: probe.durationMs || undefined,
    width: probe.video?.width,
    height: probe.video?.height,
    hasAudio: !!probe.audio,
  };
  return {
    outputs: { media },
    artifacts: [
      {
        kind: media.kind === "audio" ? "audio" : media.kind === "image" ? "image" : "video",
        path: rel(ctx, file),
        label,
      },
    ],
  };
}

async function transcode(
  ctx: ExecutorContext,
  input: MediaValue & { src: string },
  args: (inPath: string, outPath: string) => string[],
  file: string,
) {
  const inPath = refToPath(ctx, input.src);
  const outPath = path.join(ctx.dir, file);
  await ctx.ffmpeg.run(args(inPath, outPath), {
    signal: ctx.signal,
    durationMs: input.durationMs,
    onProgress: (p) => ctx.progress(p),
  });
  return outPath;
}

const extOf = (src: string) => path.extname(src).toLowerCase() || ".mp4";

const importExecutor: Executor = {
  type: "media.import",
  run: async (ctx, _inputs, p: { path: string }) => {
    if (!p.path) throw new Error("Choose a file.");
    const abs = refToPath(ctx, p.path);
    const probe = await ctx.ffmpeg.probe(abs);
    const isImage = /\.(png|jpe?g|webp|gif|svg)$/i.test(abs);
    const media: MediaValue = {
      type: "media",
      src: p.path.startsWith("project:") ? p.path : `project:${p.path}`,
      kind: isImage ? "image" : probe.video ? "video" : "audio",
      durationMs: isImage ? undefined : probe.durationMs,
      width: probe.video?.width,
      height: probe.video?.height,
      hasAudio: !!probe.audio,
    };
    return { outputs: { media } };
  },
};

const trimExecutor = mediaExec(
  "media.trim",
  async (ctx, inputs, p: { startMs: number; endMs: number }) => {
    const input = requireMedia(inputs.media);
    const file = `trimmed${extOf(input.src)}`;
    await transcode(
      ctx,
      { ...input, durationMs: p.endMs ? p.endMs - p.startMs : input.durationMs },
      (i, o) => [
        "-ss",
        (p.startMs / 1000).toFixed(3),
        ...(p.endMs ? ["-to", (p.endMs / 1000).toFixed(3)] : []),
        "-i",
        i,
        "-c:v",
        "libx264",
        "-crf",
        "18",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        o,
      ],
      file,
    );
    return mediaResult(ctx, file, "Trimmed");
  },
);

const concatExecutor = mediaExec(
  "media.concat",
  async (ctx, inputs, p: { forceReencode: boolean }) => {
    const items = (["in1", "in2", "in3", "in4"] as const)
      .map((k) => inputs[k])
      .filter(Boolean)
      .map((v) => requireMedia(v));
    if (items.length < 2) throw new Error("Connect at least two files.");
    const paths = items.map((m) => refToPath(ctx, m.src));
    const probes = await Promise.all(paths.map((f) => ctx.ffmpeg.probe(f)));
    const total = probes.reduce((s, x) => s + x.durationMs, 0);
    const file = "concat.mp4";
    const out = path.join(ctx.dir, file);
    if (!p.forceReencode && probes.every((x) => canStreamCopy(probes[0], x))) {
      const list = path.join(ctx.dir, "concat.txt");
      await writeFile(list, paths.map((f) => `file '${f.replace(/'/g, "'\\''")}'`).join("\n"));
      ctx.log("info", "The files match, joining them without converting.");
      await ctx.ffmpeg.run(
        ["-f", "concat", "-safe", "0", "-i", list, "-c", "copy", "-movflags", "+faststart", out],
        { signal: ctx.signal, durationMs: total, onProgress: ctx.progress },
      );
    } else {
      ctx.log("info", "The files differ, converting them to match the first one.");
      const w = probes[0].video?.width ?? 1920;
      const h = probes[0].video?.height ?? 1080;
      const allAudio = probes.every((x) => x.audio);
      const scaled = paths
        .map(
          (_, i) =>
            `[${i}:v]scale=${w}:${h}:force_original_aspect_ratio=decrease,pad=${w}:${h}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps=30[v${i}]`,
        )
        .join(";");
      const chain = paths.map((_, i) => `[v${i}]${allAudio ? `[${i}:a]` : ""}`).join("");
      const filter = `${scaled};${chain}concat=n=${paths.length}:v=1:a=${allAudio ? 1 : 0}[v]${allAudio ? "[a]" : ""}`;
      await ctx.ffmpeg.run(
        [
          ...paths.flatMap((f) => ["-i", f]),
          "-filter_complex",
          filter,
          "-map",
          "[v]",
          ...(allAudio ? ["-map", "[a]", "-c:a", "aac"] : []),
          "-c:v",
          "libx264",
          "-crf",
          "18",
          "-pix_fmt",
          "yuv420p",
          "-movflags",
          "+faststart",
          out,
        ],
        { signal: ctx.signal, durationMs: total, onProgress: ctx.progress },
      );
    }
    return mediaResult(ctx, file, "Concatenated");
  },
);

export function atempoChain(rate: number): string {
  const parts: number[] = [];
  let r = rate;
  while (r > 2) {
    parts.push(2);
    r /= 2;
  }
  while (r < 0.5) {
    parts.push(0.5);
    r /= 0.5;
  }
  parts.push(r);
  return parts.map((x) => `atempo=${x.toFixed(4)}`).join(",");
}

const speedExecutor = mediaExec(
  "media.speed",
  async (ctx, inputs, p: { rate: number; keepAudio: boolean }) => {
    const input = requireMedia(inputs.media);
    const file = "speed.mp4";
    await transcode(
      ctx,
      { ...input, durationMs: input.durationMs ? input.durationMs / p.rate : undefined },
      (i, o) => [
        "-i",
        i,
        "-vf",
        `setpts=PTS/${p.rate}`,
        ...(p.keepAudio && input.hasAudio !== false ? ["-af", atempoChain(p.rate)] : ["-an"]),
        "-c:v",
        "libx264",
        "-crf",
        "18",
        "-pix_fmt",
        "yuv420p",
        o,
      ],
      file,
    );
    return mediaResult(ctx, file, `Speed ${p.rate}×`);
  },
);

const reverseExecutor = mediaExec<Record<string, never>>("media.reverse", async (ctx, inputs) => {
  const input = requireMedia(inputs.media);
  const file = "reversed.mp4";
  await transcode(
    ctx,
    input,
    (i, o) => [
      "-i",
      i,
      "-vf",
      "reverse",
      ...(input.hasAudio ? ["-af", "areverse"] : ["-an"]),
      "-c:v",
      "libx264",
      "-crf",
      "18",
      "-pix_fmt",
      "yuv420p",
      o,
    ],
    file,
  );
  return mediaResult(ctx, file, "Reversed");
});

const scalePadExecutor = mediaExec(
  "media.scalePad",
  async (ctx, inputs, p: { width: number; height: number; mode: string; background: string }) => {
    const input = requireMedia(inputs.media);
    const { width: w, height: h } = p;
    const vf =
      p.mode === "pad"
        ? `scale=${w}:${h}:force_original_aspect_ratio=decrease,pad=${w}:${h}:(ow-iw)/2:(oh-ih)/2:color=${p.background},setsar=1`
        : p.mode === "crop"
          ? `scale=${w}:${h}:force_original_aspect_ratio=increase,crop=${w}:${h},setsar=1`
          : `scale=${w}:${h},setsar=1`;
    const image = input.kind === "image";
    const file = image ? "scaled.png" : "scaled.mp4";
    await transcode(
      ctx,
      input,
      (i, o) => [
        "-i",
        i,
        "-vf",
        vf,
        ...(image ? [] : ["-c:v", "libx264", "-crf", "18", "-pix_fmt", "yuv420p", "-c:a", "copy"]),
        o,
      ],
      file,
    );
    return mediaResult(ctx, file, `${w}×${h}`, image ? "image" : undefined);
  },
);

const CONVERT: Record<string, { ext: string; args: (crf: number, preset: string) => string[] }> = {
  h264: {
    ext: "mp4",
    args: (crf, preset) => [
      "-c:v",
      "libx264",
      "-crf",
      String(crf),
      "-preset",
      preset,
      "-pix_fmt",
      "yuv420p",
      "-c:a",
      "aac",
      "-movflags",
      "+faststart",
    ],
  },
  hevc: {
    ext: "mp4",
    args: (crf, preset) => [
      "-c:v",
      "libx265",
      "-crf",
      String(crf),
      "-preset",
      preset,
      "-tag:v",
      "hvc1",
      "-pix_fmt",
      "yuv420p",
      "-c:a",
      "aac",
    ],
  },
  vp9: {
    ext: "webm",
    args: (crf) => ["-c:v", "libvpx-vp9", "-crf", String(crf), "-b:v", "0", "-c:a", "libopus"],
  },
  prores: {
    ext: "mov",
    args: () => [
      "-c:v",
      "prores_ks",
      "-profile:v",
      "3",
      "-pix_fmt",
      "yuv422p10le",
      "-c:a",
      "pcm_s16le",
    ],
  },
};

const convertExecutor = mediaExec(
  "media.convert",
  async (ctx, inputs, p: { codec: string; crf: number; preset: string }) => {
    const input = requireMedia(inputs.media);
    const target = CONVERT[p.codec];
    const file = `converted.${target.ext}`;
    await transcode(ctx, input, (i, o) => ["-i", i, ...target.args(p.crf, p.preset), o], file);
    return mediaResult(ctx, file, p.codec);
  },
);

const gifExecutor = mediaExec(
  "media.gif",
  async (ctx, inputs, p: { fps: number; width: number; loop: boolean; dither: string }) => {
    const input = requireMedia(inputs.media);
    const file = "animation.gif";
    const filter = `fps=${p.fps},scale=${p.width}:-1:flags=lanczos,split[a][b];[a]palettegen=stats_mode=diff[pal];[b][pal]paletteuse=dither=${p.dither}`;
    await transcode(
      ctx,
      input,
      (i, o) => ["-i", i, "-filter_complex", filter, "-loop", p.loop ? "0" : "-1", o],
      file,
    );
    return mediaResult(ctx, file, "GIF", "image");
  },
);

const AUDIO_CODEC: Record<string, string[]> = {
  m4a: ["-c:a", "aac", "-b:a", "192k"],
  mp3: ["-c:a", "libmp3lame", "-q:a", "2"],
  wav: ["-c:a", "pcm_s16le"],
};

const extractAudioExecutor = mediaExec(
  "media.extractAudio",
  async (ctx, inputs, p: { format: string }) => {
    const input = requireMedia(inputs.media);
    const file = `audio.${p.format}`;
    await transcode(ctx, input, (i, o) => ["-i", i, "-vn", ...AUDIO_CODEC[p.format], o], file);
    return mediaResult(ctx, file, "Audio", "audio");
  },
);

const loudnormExecutor = mediaExec(
  "media.loudnorm",
  async (ctx, inputs, p: { integrated: number; truePeak: number; lra: number }) => {
    const input = requireMedia(inputs.media);
    const file = `loudnorm${input.kind === "audio" ? extOf(input.src) : ".mp4"}`;
    await transcode(
      ctx,
      input,
      (i, o) => [
        "-i",
        i,
        "-af",
        `loudnorm=I=${p.integrated}:TP=${p.truePeak}:LRA=${p.lra}`,
        ...(input.kind === "audio" ? [] : ["-c:v", "copy"]),
        "-c:a",
        "aac",
        o,
      ],
      file,
    );
    return mediaResult(ctx, file, "Loudness normalised", input.kind);
  },
);

const thumbnailExecutor = mediaExec(
  "media.thumbnail",
  async (ctx, inputs, p: { atMs: number; width: number }) => {
    const input = requireMedia(inputs.media);
    const file = "thumbnail.png";
    await ctx.ffmpeg.run(
      [
        "-ss",
        (p.atMs / 1000).toFixed(3),
        "-i",
        refToPath(ctx, input.src),
        "-frames:v",
        "1",
        "-vf",
        `scale=${p.width}:-1`,
        path.join(ctx.dir, file),
      ],
      { signal: ctx.signal },
    );
    const probe = await ctx.ffmpeg.probe(path.join(ctx.dir, file));
    return {
      outputs: {
        image: {
          type: "image",
          src: ctx.ref(file),
          width: probe.video?.width,
          height: probe.video?.height,
        },
      },
      artifacts: [{ kind: "image", path: rel(ctx, file), label: "Thumbnail" }],
    };
  },
);

const faststartExecutor = mediaExec<Record<string, never>>(
  "media.faststart",
  async (ctx, inputs) => {
    const input = requireMedia(inputs.media);
    const file = `faststart${extOf(input.src)}`;
    await transcode(
      ctx,
      input,
      (i, o) => ["-i", i, "-c", "copy", "-movflags", "+faststart", o],
      file,
    );
    return mediaResult(ctx, file, "Faststart");
  },
);

const probeExecutor = mediaExec<Record<string, never>>("media.probe", async (ctx, inputs) => {
  const input = requireMedia(inputs.media);
  const probe = await ctx.ffmpeg.probe(refToPath(ctx, input.src));
  await writeFile(path.join(ctx.dir, "probe.json"), JSON.stringify(probe.raw, null, 2));
  ctx.log(
    "info",
    `${probe.format} · ${(probe.durationMs / 1000).toFixed(2)}s · ${probe.video ? `${probe.video.codec} ${probe.video.width}×${probe.video.height} @ ${probe.video.fps.toFixed(2)}fps` : "no video"} · ${probe.audio ? `${probe.audio.codec} ${probe.audio.sampleRate}Hz` : "no audio"}`,
  );
  return {
    outputs: {
      file: { type: "file", src: ctx.ref("probe.json") },
      media: {
        ...input,
        durationMs: probe.durationMs,
        width: probe.video?.width,
        height: probe.video?.height,
        hasAudio: !!probe.audio,
      },
    },
    artifacts: [{ kind: "json", path: rel(ctx, "probe.json"), label: "Probe" }],
  };
});

export const mediaExecutors: Executor[] = [
  importExecutor,
  trimExecutor,
  concatExecutor,
  speedExecutor,
  reverseExecutor,
  scalePadExecutor,
  convertExecutor,
  gifExecutor,
  extractAudioExecutor,
  loudnormExecutor,
  thumbnailExecutor,
  faststartExecutor,
  probeExecutor,
];
