import { existsSync } from "node:fs";
import { mkdir, rename, utimes, writeFile } from "node:fs/promises";
import path from "node:path";
import { PROJECT_NAME_VAR } from "../../shared/compile/compile";
import type { NodeParams } from "../../shared/nodes/define";
import {
  renderNode,
  STILL_PRESETS,
  stillOutputNode,
  variantSize,
  type Variant,
} from "../../shared/nodes/output";
import type { TimelineSpec } from "../../shared/schema/timeline";
import type { Artifact, MediaValue } from "../../shared/schema/values";
import { defineExecutor, type ExecutorContext } from "../engine/context";
import { startAssetServer } from "../render/asset-server";
import { CODECS, hasSoundTrack } from "../media/codecs";
import { getServeUrl, renderFrame, renderTimeline } from "../render/render";
import { finishArgs, muxArgs, pictureKey, prunePictures } from "../render/picture";
import {
  chapterList,
  chapterMetadata,
  filmChapters,
  MIN_LISTED_CHAPTERS,
  type FilmChapter,
} from "../../shared/timeline/chapters";
import { refToPath } from "./util";
import { buildMixPlan, hasSound } from "../../shared/mix/plan";
import { applyMixLevels, withMasterStem } from "../../shared/mix/levels";
import { renderMix } from "../mix/render";
import { projectPaths, toPosixRel } from "../../engine/project-paths";
import { framesToMs, msToFrames } from "../../shared/timeline/time";
import { safeName } from "../../shared/safe-name";
import { projectRef } from "../../shared/asset-ref";

type RenderParams = NodeParams<typeof renderNode>;

export type Stage = readonly [from: number, to: number];

const within = ([from, to]: Stage, p: number) => from + (to - from) * p;

const MIX_STAGE: Stage = [0, 0.05];
const BUNDLE_STAGE: Stage = [0.1, 0.2];
const RENDER_STAGE: Stage = [0.2, 1];

const relativePath = (ctx: ExecutorContext, file: string) => toPosixRel(ctx.projectDir, file);

function firstArtifact(artifacts: Artifact[]): Artifact {
  const [first] = artifacts;
  if (!first) throw new Error("Nothing was exported. Pick at least one size.");
  return first;
}

export function specFor(ctx: ExecutorContext): TimelineSpec {
  const result = ctx.compile();
  const error = result.errors[ctx.node.id];
  if (error) throw new Error(error);
  const spec = result.specs[ctx.node.id];
  if (!spec) throw new Error("Connect a clip to this node.");
  return spec;
}

export async function mixed(ctx: ExecutorContext, spec: TimelineSpec, stage: Stage) {
  const plan = buildMixPlan(spec, ctx.workflow.mix);
  if (!hasSound(plan)) return applyMixLevels(spec, ctx.workflow.mix);
  const result = await renderMix(plan, {
    projectDir: ctx.projectDir,
    resolve: (ref) => refToPath(ctx, ref),
    ffmpeg: ctx.ffmpeg,
    signal: ctx.signal,
    onProgress: (p, message) => ctx.progress(within(stage, p), message),
  });
  if (!result.master) return applyMixLevels(spec, ctx.workflow.mix);
  const lufs = result.loudness?.integrated;
  const peak = result.loudness?.truePeak;
  if (lufs != null)
    ctx.log("info", `Sound: ${lufs} LUFS${peak != null ? `, peak ${peak} dBTP` : ""}`);
  return withMasterStem(spec, result.master);
}

async function prepare(ctx: ExecutorContext) {
  ctx.progress(0, "Preparing render engine");
  const serveUrl = await getServeUrl(ctx.env, (p) =>
    ctx.progress(within(BUNDLE_STAGE, p), "Bundling compositions"),
  );
  const server = await startAssetServer(ctx.projectDir);
  return { serveUrl, server };
}

type RenderJob = {
  ctx: ExecutorContext;
  serveUrl: string;
  spec: TimelineSpec;
  p: RenderParams;
  progress: (x: number) => void;
};

const renderTo = (
  { ctx, serveUrl, spec, p, progress }: RenderJob,
  output: string,
  muted: boolean,
) =>
  renderTimeline({
    env: ctx.env,
    serveUrl,
    spec,
    output,
    codec: p.codec,
    crf: p.crf,
    scale: p.scale,
    gl: p.gl,
    muted,
    signal: ctx.signal,
    onProgress: progress,
  });

async function pictureFor(job: RenderJob): Promise<string> {
  const { ctx, spec, p } = job;
  const ext = CODECS[p.codec].ext;
  const dir = projectPaths(ctx.projectDir).pictures;
  await mkdir(dir, { recursive: true });
  const key = pictureKey(spec, { codec: p.codec, crf: p.crf, scale: p.scale, gl: p.gl });
  const file = path.join(dir, `${key}.${ext}`);
  if (existsSync(file)) {
    const now = new Date();
    await utimes(file, now, now).catch(() => undefined);
    ctx.log("info", "The picture hasn't changed, so only the sound is added again.");
    job.progress(1);
    return file;
  }
  const temp = `${file}.part.${ext}`;
  await renderTo(job, temp, true);
  await rename(temp, file);
  await prunePictures(dir);
  return file;
}

type Chapters = {
  chapters: FilmChapter[];
  metadata: string | null;
  listed: boolean;
};

async function chaptersFor(
  ctx: ExecutorContext,
  spec: TimelineSpec,
  p: RenderParams,
): Promise<Chapters> {
  const chapters = p.codec === "gif" ? [] : filmChapters(spec);
  const metadata = p.chapters && chapters.length ? path.join(ctx.dir, "chapters.txt") : null;
  if (metadata) await writeFile(metadata, chapterMetadata(chapters));
  const listed = p.chapterList && chapters.length >= MIN_LISTED_CHAPTERS;
  if (chapters.length) ctx.log("info", `Chapters: ${chapters.map((c) => c.title).join(", ")}`);
  if (p.chapterList && chapters.length && !listed)
    ctx.log(
      "warn",
      `The chapter list needs at least ${MIN_LISTED_CHAPTERS} chapters of ten seconds or more, so it wasn't saved.`,
    );
  return { chapters, metadata, listed };
}

async function renderVariant(
  job: RenderJob,
  variant: Variant,
  out: string,
  chapters: Chapters,
): Promise<Artifact[]> {
  const { ctx, spec, p } = job;
  if (spec.mix && hasSoundTrack(p.codec)) {
    const sound = refToPath(ctx, spec.mix.src);
    const picture = await pictureFor(job);
    await ctx.ffmpeg.run(muxArgs(p.codec, picture, sound, out), { signal: ctx.signal });
  } else await renderTo(job, out, false);
  const { faststart } = CODECS[p.codec];
  if (faststart || chapters.metadata) {
    const tmp = `${out}.finish${path.extname(out)}`;
    await ctx.ffmpeg.run(finishArgs(out, tmp, { faststart, chapters: chapters.metadata }), {
      signal: ctx.signal,
    });
    await rename(tmp, out);
  }
  const artifacts: Artifact[] = [
    { kind: p.codec === "gif" ? "image" : "video", path: relativePath(ctx, out), label: variant },
  ];
  if (chapters.listed) {
    const list = out.replace(/\.[^.]+$/, ".youtube-chapters.txt");
    await writeFile(list, chapterList(chapters.chapters));
    artifacts.push({
      kind: "file",
      path: relativePath(ctx, list),
      label: `${variant} chapter list`,
    });
  }
  return artifacts;
}

export const renderExecutor = defineExecutor(renderNode, async (ctx, _inputs, p) => {
  const spec = await mixed(ctx, specFor(ctx), MIX_STAGE);
  const { serveUrl, server } = await prepare(ctx);
  const artifacts: Artifact[] = [];
  const base = safeName(p.fileName, { fallback: "output" });
  const chapters = await chaptersFor(ctx, spec, p);
  const { variants } = p;
  try {
    for (const [i, variant] of variants.entries()) {
      const size = variantSize(variant, ctx.workflow.settings);
      const suffix = variants.length > 1 ? `-${variant.replace(":", "x")}` : "";
      const out = path.join(ctx.outDir, `${base}${suffix}.${CODECS[p.codec].ext}`);
      ctx.log(
        "info",
        `Exporting ${variant} (${size.width}×${size.height}) as ${path.basename(out)}`,
      );
      const job: RenderJob = {
        ctx,
        serveUrl,
        spec: { ...spec, ...size, reframe: p.reframe, assets: { projectBase: server.url } },
        p,
        progress: (x) =>
          ctx.progress(within(RENDER_STAGE, (i + x) / variants.length), `Rendering ${variant}`),
      };
      artifacts.push(...(await renderVariant(job, variant, out, chapters)));
    }
  } finally {
    await server.close();
  }
  const media: MediaValue = {
    type: "media",
    src: projectRef(firstArtifact(artifacts).path),
    kind: p.codec === "gif" ? "image" : "video",
    durationMs: framesToMs(spec.durationInFrames, spec.fps),
  };
  return { outputs: { media }, artifacts };
});

export const stillOutputExecutor = defineExecutor(stillOutputNode, async (ctx, _inputs, p) => {
  const spec = specFor(ctx);
  const { serveUrl, server } = await prepare(ctx);
  const artifacts: Artifact[] = [];
  const base = safeName(ctx.vars[PROJECT_NAME_VAR], { fallback: "output" });
  try {
    for (const [i, preset] of p.presets.entries()) {
      const size = STILL_PRESETS[preset];
      const out = path.join(ctx.outDir, `${base}-${preset}.${p.format === "jpeg" ? "jpg" : "png"}`);
      ctx.progress(within(RENDER_STAGE, i / p.presets.length), `Still ${preset}`);
      await renderFrame({
        env: ctx.env,
        serveUrl,
        spec: { ...spec, ...size, reframe: "follow", assets: { projectBase: server.url } },
        frame: msToFrames(p.atMs, spec.fps, 0),
        output: out,
        format: p.format,
        gl: p.gl,
        signal: ctx.signal,
      });
      artifacts.push({ kind: "image", path: relativePath(ctx, out), label: preset });
    }
  } finally {
    await server.close();
  }
  return {
    outputs: { image: { type: "image", src: projectRef(firstArtifact(artifacts).path) } },
    artifacts,
  };
});
