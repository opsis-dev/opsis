import { existsSync } from "node:fs";
import { mkdir, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { projectRef } from "@opsis/core/asset-ref";
import { formatPreciseDuration, singleQuoted } from "@opsis/core/format";
import { formatDuration } from "@opsis/core/format";
import { findById } from "@opsis/core/lists";
import { kindFromPath } from "@opsis/core/mime";
import { loudnormFilter } from "@opsis/core/mix/graph";
import { gradeNeedsVideo, lookFileNode } from "@opsis/core/nodes/grade";
import {
  applyGradeNode,
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
} from "@opsis/core/nodes/media";
import { nodeName } from "@opsis/core/nodes/registry";
import { asValue, type MediaValue } from "@opsis/core/schema/values";
import { DEFAULT_STILL_MS } from "@opsis/core/schema/workflow";
import { writeJson } from "../../engine/json";
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
  editedFile,
  evenDown,
  EVEN_FRAME,
  FASTSTART,
  ffmpegSeconds,
  frameGrabArgs,
  gradeSoundArgs,
  gradeVideoArgs,
  PCM_16,
  soundFile,
} from "../media/codecs";
import { canStreamCopy, type ProbeResult } from "../media/ffmpeg";
import { fitFilter, gifFilter, joinFilter, speedArgs, speedSoundArgs } from "../media/filters";
import { gradeFilters } from "../media/grade";
import {
  frameBytes,
  joinReversedArgs,
  pieceLengthMs,
  REVERSE_BUDGET_BYTES,
  reverseArgs,
  reversedPartsList,
  reversePieces,
  reverseSoundArgs,
  type ReversePiece,
} from "../media/reverse";
import {
  mediaHasSound,
  projectFileRef,
  refToPath,
  rel,
  requireMedia,
  type LinkedMedia,
} from "./util";

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

const MP4_PICTURES: ReadonlySet<string> = new Set(["h264", "hevc", "av1", "vp9", "mpeg4"]);

const LAST_FRAME_MARGIN_MS = 100;

export function thumbnailTime(atMs: number, source: Pick<ProbeResult, "durationMs">): number {
  if (source.durationMs <= LAST_FRAME_MARGIN_MS) return 0;
  return Math.min(atMs, source.durationMs - LAST_FRAME_MARGIN_MS);
}

const extOf = (src: string) => path.extname(src).toLowerCase() || ".mp4";

function soundOnlyFile(probe: ProbeResult, src: string, node: string) {
  if (!probe.audio) throw new Error(`${node} works on video and sound, and this file has neither.`);
  return soundFile(extOf(src));
}

const concatList = (paths: readonly string[]) =>
  paths.map((f) => `file ${singleQuoted(f)}`).join("\n");

const PARTS_SHARE = 0.9;

type ReverseParts = { pieces: readonly ReversePiece[]; sound: boolean; durationMs: number };

async function reverseInParts(
  ctx: ExecutorContext,
  src: string,
  out: string,
  { pieces, sound, durationMs }: ReverseParts,
): Promise<void> {
  const partsDir = path.join(ctx.dir, "reverse-parts");
  await rm(partsDir, { recursive: true, force: true });
  await mkdir(partsDir, { recursive: true });
  try {
    const parts = pieces.map((piece, n) => ({
      file: path.join(partsDir, `part-${n}.mkv`),
      lengthMs: pieceLengthMs(piece, durationMs),
    }));
    let done = 0;
    for (const [n, piece] of pieces.entries()) {
      const { file, lengthMs } = parts[n];
      const share = (PARTS_SHARE * lengthMs) / durationMs;
      const message = `Reversing part ${n + 1} of ${pieces.length}`;
      const start = done;
      ctx.progress(start, message);
      await ctx.ffmpeg.run(reverseArgs(src, file, { piece, sound, soundCodec: PCM_16 }), {
        signal: ctx.signal,
        durationMs: lengthMs,
        onProgress: (p) => ctx.progress(start + p * share, message),
      });
      done += share;
    }
    const list = path.join(partsDir, "parts.txt");
    await writeFile(list, reversedPartsList(parts));
    await ctx.ffmpeg.run(joinReversedArgs(list, out, sound), {
      signal: ctx.signal,
      durationMs,
      onProgress: (p) => ctx.progress(PARTS_SHARE + p * (1 - PARTS_SHARE), "Joining the parts"),
    });
  } finally {
    await rm(partsDir, { recursive: true, force: true });
  }
}

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
    const probe = await ctx.ffmpeg.probe(refToPath(ctx, input.src));
    const moving = !!probe.video && probe.durationMs > 0;
    const out = editedFile(extOf(input.src), { video: moving, audio: !!probe.audio });
    if (!out) throw new Error("Trim works on video and sound. A still image has nothing to trim.");
    return transcodeMedia(ctx, input, {
      file: `trimmed${out.ext}`,
      label: "Trimmed",
      durationMs: p.endMs ? p.endMs - p.startMs : input.durationMs,
      args: (i, o) => [
        "-ss",
        ffmpegSeconds(p.startMs),
        ...(p.endMs ? ["-to", ffmpegSeconds(p.endMs)] : []),
        "-i",
        i,
        ...out.codecs,
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
    const parts = items.map((m, i) => {
      const still = m.kind === "image";
      return {
        still,
        video: !!probes[i].video,
        audio: !!probes[i].audio,
        durationMs: still ? (m.durationMs ?? DEFAULT_STILL_MS) : probes[i].durationMs,
      };
    });
    const total = parts.reduce((s, x) => s + x.durationMs, 0);
    const anyVideo = parts.some((x) => x.video);
    const file = anyVideo ? "concat.mp4" : "concat.m4a";
    const out = path.join(ctx.dir, file);
    const runOptions = { signal: ctx.signal, durationMs: total, onProgress: ctx.progress };
    if (!p.forceReencode && anyVideo && probes.every((x) => canStreamCopy(probes[0], x))) {
      const list = path.join(ctx.dir, "concat.txt");
      await writeFile(list, concatList(paths));
      ctx.log("info", "The files match, joining them without converting.");
      await ctx.ffmpeg.run(
        ["-f", "concat", "-safe", "0", "-i", list, "-c", "copy", ...FASTSTART, out],
        runOptions,
      );
    } else {
      ctx.log("info", "The files differ, converting them to match the first one.");
      const first = probes.find((x) => x.video)?.video;
      const join = joinFilter(parts, {
        width: first?.width ?? ctx.workflow.settings.width,
        height: first?.height ?? ctx.workflow.settings.height,
        fps: first?.fps || ctx.workflow.settings.fps,
      });
      await ctx.ffmpeg.run(
        [
          ...paths.flatMap((f, i) => {
            const part = parts[i];
            return part.still
              ? ["-loop", "1", "-t", ffmpegSeconds(part.durationMs), "-i", f]
              : ["-i", f];
          }),
          "-filter_complex",
          join.filter,
          ...(join.video ? ["-map", "[v]", ...EDIT_VIDEO] : []),
          ...(join.audio ? ["-map", "[a]", ...EDIT_AUDIO] : []),
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
    const label = `Speed ${p.rate}×`;
    const durationMs = input.durationMs ? input.durationMs / p.rate : undefined;
    if (!probe.video) {
      const out = soundOnlyFile(probe, input.src, "Change speed");
      if (!p.keepAudio)
        throw new Error("This file is only sound, so turning off Keep the sound leaves nothing.");
      return transcodeMedia(ctx, input, {
        file: `speed${out.ext}`,
        label,
        durationMs,
        args: (i, o) => speedSoundArgs(i, o, { rate: p.rate, codecs: out.codecs }),
      });
    }
    const fps = probe.video.fps || ctx.workflow.settings.fps;
    const sound = p.keepAudio && (await mediaHasSound(ctx, input));
    return transcodeMedia(ctx, input, {
      file: "speed.mp4",
      label,
      durationMs,
      args: (i, o) => speedArgs(i, o, { rate: p.rate, fps, sound }),
    });
  }),

  defineExecutor(reverseNode, async (ctx, inputs) => {
    const input = requireMedia(inputs.media);
    const src = refToPath(ctx, input.src);
    const probe = await ctx.ffmpeg.probe(src);
    const label = "Reversed";
    if (!probe.video) {
      const out = soundOnlyFile(probe, input.src, "Reverse");
      return transcodeMedia(ctx, input, {
        file: `reversed${out.ext}`,
        label,
        args: (i, o) => reverseSoundArgs(i, o, out.codecs),
      });
    }
    const sound = await mediaHasSound(ctx, input);
    const pieces = reversePieces({
      durationMs: probe.durationMs,
      fps: probe.video.fps || ctx.workflow.settings.fps,
      frameBytes: frameBytes(probe.video),
      budgetBytes: REVERSE_BUDGET_BYTES,
    });
    const file = "reversed.mp4";
    if (pieces.length === 1) {
      return transcodeMedia(ctx, input, {
        file,
        label,
        args: (i, o) => reverseArgs(i, o, { sound }),
      });
    }
    ctx.log("info", `Reversing in ${pieces.length} parts to keep memory low.`);
    await reverseInParts(ctx, src, path.join(ctx.dir, file), {
      pieces,
      sound,
      durationMs: probe.durationMs,
    });
    return mediaResult(ctx, file, label);
  }),

  defineExecutor(scalePadNode, async (ctx, inputs, p) => {
    const input = requireMedia(inputs.media);
    const probe = await ctx.ffmpeg.probe(refToPath(ctx, input.src));
    if (!probe.video)
      throw new Error("Resize works on video and images, and this file is only sound.");
    const image = input.kind === "image";
    return transcodeMedia(ctx, input, {
      file: image ? "scaled.png" : "scaled.mp4",
      label: `${p.width}×${p.height}`,
      args: (i, o) => [
        "-i",
        i,
        "-vf",
        image
          ? fitFilter(p)
          : fitFilter({ ...p, width: evenDown(p.width), height: evenDown(p.height) }),
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

  defineExecutor(applyGradeNode, async (ctx, inputs, p) => {
    const input = requireMedia(inputs.media);
    const entries = asValue(inputs.grade, "grade")?.entries;
    if (!entries) throw new Error("Connect a grade.");
    for (const entry of entries) {
      if (entry.type !== lookFileNode.type) continue;
      const look = lookFileNode.params.parse(entry.params);
      if (look.strength === 0) continue;
      const name = nodeName(findById(ctx.workflow.nodes, entry.nodeId) ?? { type: entry.type });
      if (!look.file) throw new Error(`Choose a look file for “${name}”.`);
      if (!existsSync(refToPath(ctx, projectRef(look.file))))
        throw new Error(`The look file for “${name}” isn't in the project.`);
    }
    const src = refToPath(ctx, input.src);
    const probe = await ctx.ffmpeg.probe(src);
    if (!probe.video) throw new Error("This file has no picture to grade.");
    const still = input.kind === "image";
    const { chain, extraInputs } = gradeFilters(
      entries,
      { width: probe.video.width, height: probe.video.height, still },
      (file) => refToPath(ctx, projectRef(file)),
    );
    if (still && gradeNeedsVideo(entries))
      ctx.log("warn", "Motion trails and film grain are left out of a still image.");
    const file = still ? "graded.png" : `graded.${CODECS[p.codec].ext}`;
    return transcodeMedia(ctx, input, {
      file,
      label: "Graded file",
      args: (i, o) => [
        "-i",
        i,
        ...extraInputs.flatMap((f) => ["-i", f]),
        "-filter_complex",
        chain,
        "-map",
        "[out]",
        ...(still
          ? ["-frames:v", "1"]
          : [
              ...(probe.audio ? ["-map", "0:a?"] : []),
              ...gradeVideoArgs(p.codec, p.crf, p.preset),
              ...gradeSoundArgs(p.codec, probe.audio?.codec),
              ...(CODECS[p.codec].faststart ? FASTSTART : []),
            ]),
        o,
      ],
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
    if (!(await mediaHasSound(ctx, input))) throw new Error("This file has no sound to take out.");
    return transcodeMedia(ctx, input, {
      file: `audio.${p.format}`,
      label: "Audio",
      args: (i, o) => ["-i", i, "-vn", ...AUDIO_FORMATS[p.format], o],
    });
  }),

  defineExecutor(loudnormNode, async (ctx, inputs, p) => {
    const input = requireMedia(inputs.media);
    const format = input.kind === "audio" ? audioFormatFor(extOf(input.src).slice(1)) : null;
    const { audio, video } = await ctx.ffmpeg.probe(refToPath(ctx, input.src));
    if (!audio) throw new Error("This file has no sound to even out.");
    const picture =
      video && MP4_PICTURES.has(video.codec)
        ? ["-c:v", "copy"]
        : ["-vf", EVEN_FRAME, ...EDIT_VIDEO];
    return transcodeMedia(ctx, input, {
      file: `loudnorm.${format ?? "mp4"}`,
      label: "Loudness normalised",
      args: (i, o) => [
        "-i",
        i,
        "-af",
        loudnormFilter(p.integrated, p.truePeak, p.lra),
        "-ar",
        String(audio.sampleRate),
        ...(format ? AUDIO_FORMATS[format] : [...picture, ...EDIT_AUDIO]),
        o,
      ],
    });
  }),

  defineExecutor(thumbnailNode, async (ctx, inputs, p) => {
    const input = requireMedia(inputs.media);
    const src = refToPath(ctx, input.src);
    const source = await ctx.ffmpeg.probe(src);
    if (!source.video) throw new Error("This file has no picture to take a thumbnail from.");
    const atMs = thumbnailTime(p.atMs, source);
    if (atMs < p.atMs && input.kind !== "image")
      ctx.log(
        "warn",
        `The file is only ${formatDuration(source.durationMs)} long, so the thumbnail is from ${formatDuration(atMs)}.`,
      );
    const file = "thumbnail.png";
    await ctx.ffmpeg.run(frameGrabArgs(src, path.join(ctx.dir, file), { atMs, width: p.width }), {
      signal: ctx.signal,
    });
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
        info: { type: "file-info", src: projectFileRef(ctx, "probe.json") },
        media: mediaValueFrom(probe, input.src, input.kind),
      },
      artifacts: [{ kind: "json", path: rel(ctx, "probe.json"), label: "Probe" }],
    };
  }),
];
