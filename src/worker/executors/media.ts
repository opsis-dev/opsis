import { writeFile } from "node:fs/promises";
import path from "node:path";
import { writeJson } from "../../engine/json";
import { projectRef } from "../../shared/asset-ref";
import { singleQuoted } from "../../shared/format";
import { formatPreciseDuration } from "../../shared/format";
import { kindFromPath } from "../../shared/mime";
import { loudnormFilter } from "../../shared/mix/graph";
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
import {
  defineExecutor,
  type Executor,
  type ExecutorContext,
  type ExecutorOutput,
} from "../engine/context";
import {
  AUDIO_FORMATS,
  audioFormatFor,
  CODECS,
  convertArgs,
  EDIT_AUDIO,
  EDIT_VIDEO,
  FASTSTART,
  ffmpegSeconds,
  frameGrabArgs,
} from "../media/codecs";
import { canStreamCopy, type ProbeResult } from "../media/ffmpeg";
import { fitFilter, gifFilter, joinFilter, speedArgs } from "../media/filters";
import { projectFileRef, refToPath, rel, requireMedia, type LinkedMedia } from "./util";

const mediaKind = (probe: ProbeResult, file: string): MediaValue["kind"] => {
  const named = kindFromPath(file);
  return named === "video" && !probe.video ? "audio" : named;
};

const mediaValueFrom = (
  probe: ProbeResult,
  src: string,
  kind: MediaValue["kind"] = mediaKind(probe, src),
): MediaValue => ({
  type: "media",
  src,
  kind,
  durationMs: probe.durationMs || undefined,
  width: probe.video?.width,
  height: probe.video?.height,
  hasAudio: !!probe.audio,
});

async function mediaResult(
  ctx: ExecutorContext,
  file: string,
  label: string,
): Promise<ExecutorOutput> {
  const probe = await ctx.ffmpeg.probe(path.join(ctx.dir, file));
  const media = mediaValueFrom(probe, projectFileRef(ctx, file));
  return {
    outputs: { media },
    artifacts: [{ kind: media.kind, path: rel(ctx, file), label }],
  };
}

type Transcode = {
  file: string;
  label: string;
  durationMs?: number;
  args: (inPath: string, outPath: string) => string[];
};

async function transcodeMedia(
  ctx: ExecutorContext,
  input: LinkedMedia,
  { file, label, durationMs = input.durationMs, args }: Transcode,
): Promise<ExecutorOutput> {
  await ctx.ffmpeg.run(args(refToPath(ctx, input.src), path.join(ctx.dir, file)), {
    signal: ctx.signal,
    durationMs,
    onProgress: (p) => ctx.progress(p),
  });
  return mediaResult(ctx, file, label);
}

const extOf = (src: string) => path.extname(src).toLowerCase() || ".mp4";

const concatList = (paths: readonly string[]) =>
  paths.map((f) => `file ${singleQuoted(f)}`).join("\n");

function probeSummary(probe: ProbeResult): string {
  const { video, audio } = probe;
  const picture = video
    ? `${video.codec} ${video.width}×${video.height} @ ${video.fps.toFixed(2)}fps`
    : "no video";
  const sound = audio ? `${audio.codec} ${audio.sampleRate}Hz` : "no audio";
  return `${probe.format} · ${formatPreciseDuration(probe.durationMs)} · ${picture} · ${sound}`;
}

export const mediaExecutors: Executor[] = [
  defineExecutor(importMediaNode, async (ctx, _inputs, p) => {
    if (!p.path) throw new Error("Choose a file.");
    const probe = await ctx.ffmpeg.probe(refToPath(ctx, p.path));
    const media = mediaValueFrom(probe, projectRef(p.path));
    return {
      outputs: { media: media.kind === "image" ? { ...media, durationMs: undefined } : media },
    };
  }),

  defineExecutor(mediaTrimNode, async (ctx, inputs, p) => {
    const input = requireMedia(inputs.media);
    return transcodeMedia(ctx, input, {
      file: `trimmed${extOf(input.src)}`,
      label: "Trimmed",
      durationMs: p.endMs ? p.endMs - p.startMs : input.durationMs,
      args: (i, o) => [
        "-ss",
        ffmpegSeconds(p.startMs),
        ...(p.endMs ? ["-to", ffmpegSeconds(p.endMs)] : []),
        "-i",
        i,
        ...EDIT_VIDEO,
        ...EDIT_AUDIO,
        o,
      ],
    });
  }),

  defineExecutor(concatNode, async (ctx, inputs, p) => {
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
    const runOptions = { signal: ctx.signal, durationMs: total, onProgress: ctx.progress };
    if (!p.forceReencode && probes.every((x) => canStreamCopy(probes[0], x))) {
      const list = path.join(ctx.dir, "concat.txt");
      await writeFile(list, concatList(paths));
      ctx.log("info", "The files match, joining them without converting.");
      await ctx.ffmpeg.run(
        ["-f", "concat", "-safe", "0", "-i", list, "-c", "copy", ...FASTSTART, out],
        runOptions,
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
        runOptions,
      );
    }
    return mediaResult(ctx, file, "Concatenated");
  }),

  defineExecutor(mediaSpeedNode, async (ctx, inputs, p) => {
    const input = requireMedia(inputs.media);
    const probe = await ctx.ffmpeg.probe(refToPath(ctx, input.src));
    const fps = probe.video?.fps || ctx.workflow.settings.fps;
    return transcodeMedia(ctx, input, {
      file: "speed.mp4",
      label: `Speed ${p.rate}×`,
      durationMs: input.durationMs ? input.durationMs / p.rate : undefined,
      args: (i, o) =>
        speedArgs(i, o, { rate: p.rate, fps, sound: p.keepAudio && input.hasAudio !== false }),
    });
  }),

  defineExecutor(reverseNode, async (ctx, inputs) => {
    const input = requireMedia(inputs.media);
    return transcodeMedia(ctx, input, {
      file: "reversed.mp4",
      label: "Reversed",
      args: (i, o) => [
        "-i",
        i,
        "-vf",
        "reverse",
        ...(input.hasAudio ? ["-af", "areverse"] : ["-an"]),
        ...EDIT_VIDEO,
        o,
      ],
    });
  }),

  defineExecutor(scalePadNode, async (ctx, inputs, p) => {
    const input = requireMedia(inputs.media);
    const image = input.kind === "image";
    return transcodeMedia(ctx, input, {
      file: image ? "scaled.png" : "scaled.mp4",
      label: `${p.width}×${p.height}`,
      args: (i, o) => [
        "-i",
        i,
        "-vf",
        fitFilter(p),
        ...(image ? [] : [...EDIT_VIDEO, "-c:a", "copy"]),
        o,
      ],
    });
  }),

  defineExecutor(convertNode, async (ctx, inputs, p) => {
    const input = requireMedia(inputs.media);
    return transcodeMedia(ctx, input, {
      file: `converted.${CODECS[p.codec].ext}`,
      label: p.codec,
      args: (i, o) => ["-i", i, ...convertArgs(p.codec, p.crf, p.preset), o],
    });
  }),

  defineExecutor(gifNode, async (ctx, inputs, p) => {
    const input = requireMedia(inputs.media);
    return transcodeMedia(ctx, input, {
      file: "animation.gif",
      label: "GIF",
      args: (i, o) => ["-i", i, "-filter_complex", gifFilter(p), "-loop", p.loop ? "0" : "-1", o],
    });
  }),

  defineExecutor(extractAudioNode, async (ctx, inputs, p) => {
    const input = requireMedia(inputs.media);
    return transcodeMedia(ctx, input, {
      file: `audio.${p.format}`,
      label: "Audio",
      args: (i, o) => ["-i", i, "-vn", ...AUDIO_FORMATS[p.format], o],
    });
  }),

  defineExecutor(loudnormNode, async (ctx, inputs, p) => {
    const input = requireMedia(inputs.media);
    const format = input.kind === "audio" ? audioFormatFor(extOf(input.src).slice(1)) : null;
    const { audio } = await ctx.ffmpeg.probe(refToPath(ctx, input.src));
    return transcodeMedia(ctx, input, {
      file: `loudnorm.${format ?? "mp4"}`,
      label: "Loudness normalised",
      args: (i, o) => [
        "-i",
        i,
        "-af",
        loudnormFilter(p.integrated, p.truePeak, p.lra),
        ...(audio ? ["-ar", String(audio.sampleRate)] : []),
        ...(format ? AUDIO_FORMATS[format] : ["-c:v", "copy", ...EDIT_AUDIO]),
        o,
      ],
    });
  }),

  defineExecutor(thumbnailNode, async (ctx, inputs, p) => {
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
  }),

  defineExecutor(faststartNode, async (ctx, inputs) => {
    const input = requireMedia(inputs.media);
    return transcodeMedia(ctx, input, {
      file: `faststart${extOf(input.src)}`,
      label: "Web-ready",
      args: (i, o) => ["-i", i, "-c", "copy", ...FASTSTART, o],
    });
  }),

  defineExecutor(probeNode, async (ctx, inputs) => {
    const input = requireMedia(inputs.media);
    const probe = await ctx.ffmpeg.probe(refToPath(ctx, input.src));
    await writeJson(path.join(ctx.dir, "probe.json"), probe.raw);
    ctx.log("info", probeSummary(probe));
    return {
      outputs: {
        file: { type: "file", src: projectFileRef(ctx, "probe.json") },
        media: mediaValueFrom(probe, input.src, input.kind),
      },
      artifacts: [{ kind: "json", path: rel(ctx, "probe.json"), label: "Probe" }],
    };
  }),
];
