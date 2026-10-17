import { writeFile } from "node:fs/promises";
import path from "node:path";
import {
  concatNode,
  convertNode,
  extractAudioNode,
  faststartNode,
  gifNode,
  importMediaNode,
  loudnormNode,
  mediaSpeedNode,
  mediaTrimNode,
  probeNode,
  reverseNode,
  scalePadNode,
  thumbnailNode,
} from "../../shared/nodes/media";
import type { MediaValue } from "../../shared/schema/values";
import { singleQuoted } from "../../shared/format";
import {
  defineExecutor,
  type Executor,
  type ExecutorContext,
  type ExecutorOutput,
} from "../engine/context";
import { canStreamCopy, type ProbeResult } from "../media/ffmpeg";
import {
  AUDIO_FORMATS,
  CONVERT,
  EDIT_AUDIO,
  EDIT_VIDEO,
  FASTSTART,
  frameGrabArgs,
} from "../media/codecs";
import { joinFilter, speedArgs } from "../media/filters";
import { projectFileRef, refToPath, rel, requireMedia } from "./util";
import { projectRef } from "../../shared/asset-ref";
import { formatPreciseDuration } from "../../shared/format";
import { IMAGE_EXTENSIONS } from "../../shared/mime";
import { writeJson } from "../../engine/json";

const probedKind = (probe: ProbeResult): MediaValue["kind"] =>
  probe.video ? (probe.durationMs > 0 ? "video" : "image") : "audio";

type MediaShape = { kind: MediaValue["kind"]; durationMs: number | undefined };

const probedShape = (probe: ProbeResult, kind = probedKind(probe)): MediaShape => ({
  kind,
  durationMs: probe.durationMs || undefined,
});

const mediaValueFrom = (probe: ProbeResult, src: string, shape: MediaShape): MediaValue => ({
  type: "media",
  src,
  ...shape,
  width: probe.video?.width,
  height: probe.video?.height,
  hasAudio: !!probe.audio,
});

async function mediaResult(
  ctx: ExecutorContext,
  file: string,
  label: string,
  kind?: MediaValue["kind"],
): Promise<ExecutorOutput> {
  const probe = await ctx.ffmpeg.probe(path.join(ctx.dir, file));
  const media = mediaValueFrom(probe, projectFileRef(ctx, file), probedShape(probe, kind));
  return {
    outputs: { media },
    artifacts: [{ kind: media.kind, path: rel(ctx, file), label }],
  };
}

async function transcode(
  ctx: ExecutorContext,
  src: string,
  durationMs: number | undefined,
  args: (inPath: string, outPath: string) => string[],
  file: string,
) {
  const inPath = refToPath(ctx, src);
  const outPath = path.join(ctx.dir, file);
  await ctx.ffmpeg.run(args(inPath, outPath), {
    signal: ctx.signal,
    durationMs,
    onProgress: (p) => ctx.progress(p),
  });
  return outPath;
}

const isImageFile = (file: string) =>
  IMAGE_EXTENSIONS.has(path.extname(file).slice(1).toLowerCase());

const extOf = (src: string) => path.extname(src).toLowerCase() || ".mp4";

const importExecutor = defineExecutor(importMediaNode, async (ctx, _inputs, p) => {
  if (!p.path) throw new Error("Choose a file.");
  const abs = refToPath(ctx, p.path);
  const probe = await ctx.ffmpeg.probe(abs);
  const src = projectRef(p.path);
  const kind = isImageFile(abs) ? "image" : probe.video ? "video" : "audio";
  const media = mediaValueFrom(probe, src, {
    kind,
    durationMs: kind === "image" ? undefined : probe.durationMs,
  });
  return { outputs: { media } };
});

const trimExecutor = defineExecutor(mediaTrimNode, async (ctx, inputs, p) => {
  const input = requireMedia(inputs.media);
  const file = `trimmed${extOf(input.src)}`;
  await transcode(
    ctx,
    input.src,
    p.endMs ? p.endMs - p.startMs : input.durationMs,
    (i, o) => [
      "-ss",
      (p.startMs / 1000).toFixed(3),
      ...(p.endMs ? ["-to", (p.endMs / 1000).toFixed(3)] : []),
      "-i",
      i,
      ...EDIT_VIDEO,
      ...EDIT_AUDIO,
      o,
    ],
    file,
  );
  return mediaResult(ctx, file, "Trimmed");
});

const concatExecutor = defineExecutor(concatNode, async (ctx, inputs, p) => {
  const items = Object.keys(concatNode.inputs)
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
    await writeFile(list, paths.map((f) => `file ${singleQuoted(f)}`).join("\n"));
    ctx.log("info", "The files match, joining them without converting.");
    await ctx.ffmpeg.run(
      ["-f", "concat", "-safe", "0", "-i", list, "-c", "copy", ...FASTSTART, out],
      { signal: ctx.signal, durationMs: total, onProgress: ctx.progress },
    );
  } else {
    ctx.log("info", "The files differ, converting them to match the first one.");
    const allAudio = probes.every((x) => x.audio);
    const filter = joinFilter(paths.length, {
      width: probes[0].video?.width ?? ctx.workflow.settings.width,
      height: probes[0].video?.height ?? ctx.workflow.settings.height,
      fps: probes[0].video?.fps || ctx.workflow.settings.fps,
      sound: allAudio,
    });
    await ctx.ffmpeg.run(
      [
        ...paths.flatMap((f) => ["-i", f]),
        "-filter_complex",
        filter,
        "-map",
        "[v]",
        ...(allAudio ? ["-map", "[a]", ...EDIT_AUDIO] : []),
        ...EDIT_VIDEO,
        ...FASTSTART,
        out,
      ],
      { signal: ctx.signal, durationMs: total, onProgress: ctx.progress },
    );
  }
  return mediaResult(ctx, file, "Concatenated");
});

const speedExecutor = defineExecutor(mediaSpeedNode, async (ctx, inputs, p) => {
  const input = requireMedia(inputs.media);
  const file = "speed.mp4";
  const probe = await ctx.ffmpeg.probe(refToPath(ctx, input.src));
  const fps = probe.video?.fps || ctx.workflow.settings.fps;
  await transcode(
    ctx,
    input.src,
    input.durationMs ? input.durationMs / p.rate : undefined,
    (i, o) =>
      speedArgs(i, o, { rate: p.rate, fps, sound: p.keepAudio && input.hasAudio !== false }),
    file,
  );
  return mediaResult(ctx, file, `Speed ${p.rate}×`);
});

const reverseExecutor = defineExecutor(reverseNode, async (ctx, inputs) => {
  const input = requireMedia(inputs.media);
  const file = "reversed.mp4";
  await transcode(
    ctx,
    input.src,
    input.durationMs,
    (i, o) => [
      "-i",
      i,
      "-vf",
      "reverse",
      ...(input.hasAudio ? ["-af", "areverse"] : ["-an"]),
      ...EDIT_VIDEO,
      o,
    ],
    file,
  );
  return mediaResult(ctx, file, "Reversed");
});

const scalePadExecutor = defineExecutor(scalePadNode, async (ctx, inputs, p) => {
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
    input.src,
    input.durationMs,
    (i, o) => ["-i", i, "-vf", vf, ...(image ? [] : [...EDIT_VIDEO, "-c:a", "copy"]), o],
    file,
  );
  return mediaResult(ctx, file, `${w}×${h}`, image ? "image" : undefined);
});

const convertExecutor = defineExecutor(convertNode, async (ctx, inputs, p) => {
  const input = requireMedia(inputs.media);
  const target = CONVERT[p.codec];
  const file = `converted.${target.ext}`;
  await transcode(
    ctx,
    input.src,
    input.durationMs,
    (i, o) => ["-i", i, ...target.args(p.crf, p.preset), o],
    file,
  );
  return mediaResult(ctx, file, p.codec);
});

const gifExecutor = defineExecutor(gifNode, async (ctx, inputs, p) => {
  const input = requireMedia(inputs.media);
  const file = "animation.gif";
  const filter = `fps=${p.fps},scale=${p.width}:-1:flags=lanczos,split[a][b];[a]palettegen=stats_mode=diff[pal];[b][pal]paletteuse=dither=${p.dither}`;
  await transcode(
    ctx,
    input.src,
    input.durationMs,
    (i, o) => ["-i", i, "-filter_complex", filter, "-loop", p.loop ? "0" : "-1", o],
    file,
  );
  return mediaResult(ctx, file, "GIF", "image");
});

const extractAudioExecutor = defineExecutor(extractAudioNode, async (ctx, inputs, p) => {
  const input = requireMedia(inputs.media);
  const file = `audio.${p.format}`;
  await transcode(
    ctx,
    input.src,
    input.durationMs,
    (i, o) => ["-i", i, "-vn", ...AUDIO_FORMATS[p.format], o],
    file,
  );
  return mediaResult(ctx, file, "Audio", "audio");
});

const loudnormExecutor = defineExecutor(loudnormNode, async (ctx, inputs, p) => {
  const input = requireMedia(inputs.media);
  const file = `loudnorm${input.kind === "audio" ? extOf(input.src) : ".mp4"}`;
  await transcode(
    ctx,
    input.src,
    input.durationMs,
    (i, o) => [
      "-i",
      i,
      "-af",
      `loudnorm=I=${p.integrated}:TP=${p.truePeak}:LRA=${p.lra}`,
      ...(input.kind === "audio" ? [] : ["-c:v", "copy"]),
      ...EDIT_AUDIO,
      o,
    ],
    file,
  );
  return mediaResult(ctx, file, "Loudness normalised", input.kind);
});

const thumbnailExecutor = defineExecutor(thumbnailNode, async (ctx, inputs, p) => {
  const input = requireMedia(inputs.media);
  const file = "thumbnail.png";
  await ctx.ffmpeg.run(
    frameGrabArgs(refToPath(ctx, input.src), path.join(ctx.dir, file), {
      atMs: p.atMs,
      width: p.width,
    }),
    { signal: ctx.signal },
  );
  const probe = await ctx.ffmpeg.probe(path.join(ctx.dir, file));
  return {
    outputs: {
      image: {
        type: "image",
        src: projectFileRef(ctx, file),
        width: probe.video?.width,
        height: probe.video?.height,
      },
    },
    artifacts: [{ kind: "image", path: rel(ctx, file), label: "Thumbnail" }],
  };
});

const faststartExecutor = defineExecutor(faststartNode, async (ctx, inputs) => {
  const input = requireMedia(inputs.media);
  const file = `faststart${extOf(input.src)}`;
  await transcode(
    ctx,
    input.src,
    input.durationMs,
    (i, o) => ["-i", i, "-c", "copy", ...FASTSTART, o],
    file,
  );
  return mediaResult(ctx, file, "Web-ready");
});

const probeExecutor = defineExecutor(probeNode, async (ctx, inputs) => {
  const input = requireMedia(inputs.media);
  const probe = await ctx.ffmpeg.probe(refToPath(ctx, input.src));
  await writeJson(path.join(ctx.dir, "probe.json"), probe.raw);
  ctx.log(
    "info",
    `${probe.format} · ${formatPreciseDuration(probe.durationMs)} · ${probe.video ? `${probe.video.codec} ${probe.video.width}×${probe.video.height} @ ${probe.video.fps.toFixed(2)}fps` : "no video"} · ${probe.audio ? `${probe.audio.codec} ${probe.audio.sampleRate}Hz` : "no audio"}`,
  );
  return {
    outputs: {
      file: { type: "file", src: projectFileRef(ctx, "probe.json") },
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
