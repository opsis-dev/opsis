import { existsSync } from "node:fs";
import { mkdir, rename, utimes, writeFile } from "node:fs/promises";
import path from "node:path";
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
import { mixed, specFor, type Stage } from "./film-spec";
import { lerp } from "../../shared/math";
import { projectPaths, toPosixRel } from "../../engine/project-paths";
import { framesToMs, msToFrames } from "../../shared/timeline/time";
import { safeName } from "../../shared/safe-name";
import { projectRef } from "../../shared/asset-ref";

type RenderParams = NodeParams<typeof renderNode>;

const MIX_STAGE: Stage = [0, 0.05];
const BUNDLE_STAGE: Stage = [0.1, 0.2];
const RENDER_STAGE: Stage = [0.2, 1];

function firstArtifact(artifacts: Artifact[]): Artifact {
  const [first] = artifacts;
  if (!first) throw new Error("Nothing was exported. Pick at least one size.");
  return first;
}

type RenderServer = { serveUrl: string; projectBase: string };

async function withRenderServer<T>(
  ctx: ExecutorContext,
  render: (server: RenderServer) => Promise<T>,
): Promise<T> {
  ctx.progress(0, "Preparing render engine");
  const serveUrl = await getServeUrl(ctx.env, (p) =>
    ctx.progress(lerp(...BUNDLE_STAGE, p), "Bundling compositions"),
  );
  const server = await startAssetServer(ctx.projectDir);
  try {
    return await render({ serveUrl, projectBase: server.url });
  } finally {
    await server.close();
  }
}

type VariantJob = {
  ctx: ExecutorContext;
  serveUrl: string;
  spec: TimelineSpec;
  p: RenderParams;
  progress: (x: number) => void;
};

const renderTo = (
  { ctx, serveUrl, spec, p, progress }: VariantJob,
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

async function pictureFor(job: VariantJob): Promise<string> {
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
  job: VariantJob,
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
    {
      kind: p.codec === "gif" ? "image" : "video",
      path: toPosixRel(ctx.projectDir, out),
      label: variant,
    },
  ];
  if (chapters.listed) {
    const list = out.replace(/\.[^.]+$/, ".youtube-chapters.txt");
    await writeFile(list, chapterList(chapters.chapters));
    artifacts.push({
      kind: "file",
      path: toPosixRel(ctx.projectDir, list),
      label: `${variant} chapter list`,
    });
  }
  return artifacts;
}

export const renderExecutor = defineExecutor(renderNode, async (ctx, _inputs, p) => {
  const spec = await mixed(ctx, specFor(ctx), MIX_STAGE);
  const base = safeName(p.fileName, { fallback: "output" });
  const { variants } = p;
  const artifacts = await withRenderServer(ctx, async ({ serveUrl, projectBase }) => {
    const chapters = await chaptersFor(ctx, spec, p);
    const made: Artifact[] = [];
    for (const [i, variant] of variants.entries()) {
      const size = variantSize(variant, ctx.workflow.settings);
      const suffix = variants.length > 1 ? `-${variant.replace(":", "x")}` : "";
      const out = path.join(ctx.outDir, `${base}${suffix}.${CODECS[p.codec].ext}`);
      ctx.log(
        "info",
        `Exporting ${variant} (${size.width}×${size.height}) as ${path.basename(out)}`,
      );
      const job: VariantJob = {
        ctx,
        serveUrl,
        spec: { ...spec, ...size, reframe: p.reframe, assets: { projectBase } },
        p,
        progress: (x) =>
          ctx.progress(lerp(...RENDER_STAGE, (i + x) / variants.length), `Rendering ${variant}`),
      };
      made.push(...(await renderVariant(job, variant, out, chapters)));
    }
    return made;
  });
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
  const base = safeName(p.fileName, { fallback: "output" });
  const artifacts = await withRenderServer(ctx, async ({ serveUrl, projectBase }) => {
    const made: Artifact[] = [];
    for (const [i, preset] of p.presets.entries()) {
      const size = STILL_PRESETS[preset];
      const out = path.join(ctx.outDir, `${base}-${preset}.${p.format === "jpeg" ? "jpg" : "png"}`);
      ctx.progress(lerp(...RENDER_STAGE, i / p.presets.length), `Still ${preset}`);
      await renderFrame({
        env: ctx.env,
        serveUrl,
        spec: { ...spec, ...size, reframe: "follow", assets: { projectBase } },
        frame: msToFrames(p.atMs, spec.fps, 0),
        output: out,
        format: p.format,
        gl: p.gl,
        signal: ctx.signal,
      });
      made.push({ kind: "image", path: toPosixRel(ctx.projectDir, out), label: preset });
    }
    return made;
  });
  return {
    outputs: { image: { type: "image", src: projectRef(firstArtifact(artifacts).path) } },
    artifacts,
  };
});
