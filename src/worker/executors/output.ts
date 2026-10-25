import { writeFile } from "node:fs/promises";
import path from "node:path";
import { projectRef } from "@opsis/core/asset-ref";
import { lerp } from "@opsis/core/math";
import type { NodeParams } from "@opsis/core/nodes/define";
import {
  exportBaseName,
  renderNode,
  STILL_PRESETS,
  stillFileNames,
  stillOutputNode,
  videoFileNames,
  variantSize,
  type Variant,
} from "@opsis/core/nodes/output";
import type { TimelineSpec } from "@opsis/core/schema/timeline";
import type { Artifact, MediaValue } from "@opsis/core/schema/values";
import {
  chapterList,
  chapterMetadata,
  filmChapters,
  MIN_LISTED_CHAPTERS,
  type FilmChapter,
} from "@opsis/core/timeline/chapters";
import { framesToMs, msToFrames } from "@opsis/core/timeline/time";
import { renderFrame, renderTimeline } from "@opsis/sidecar/render";
import { writeViaTemp } from "../../engine/atomic";
import { toPosixRel } from "../../engine/project-paths";
import { defineExecutor, type ExecutorContext } from "../engine/context";
import { CODECS, hasSoundTrack, kindOfCodec } from "../media/codecs";
import { startAssetServer } from "../render/asset-server";
import { cachedPicture, finishArgs, muxArgs, pictureKey } from "../render/picture";
import { getServeUrl, renderTarget } from "../render/remotion";
import { mixed, specFor, type Stage } from "./film-spec";
import { refToPath } from "./util";

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
  { muted }: { muted: boolean },
) =>
  renderTimeline(
    {
      ...renderTarget(ctx.env, serveUrl, spec, p.gl),
      output,
      codec: p.codec,
      crf: CODECS[p.codec].crf ? p.crf : undefined,
      scale: p.scale,
      muted,
    },
    { signal: ctx.signal, onProgress: progress },
  );

async function pictureFor(job: VariantJob): Promise<string> {
  const { ctx, spec, p } = job;
  const key = pictureKey(
    spec,
    { codec: p.codec, crf: p.crf, scale: p.scale, gl: p.gl },
    ctx.env.appVersion,
  );
  const picture = await cachedPicture(ctx.projectDir, `${key}.${CODECS[p.codec].ext}`, (temp) =>
    renderTo(job, temp, { muted: true }),
  );
  if (picture.reused) {
    ctx.log("info", "The picture hasn't changed, so only the sound is added again.");
    job.progress(1);
  }
  return picture.file;
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
  const { codec } = p;
  if (spec.mix && hasSoundTrack(codec)) {
    const sound = refToPath(ctx, spec.mix.src);
    const picture = await pictureFor(job);
    await writeViaTemp(out, (temp) =>
      ctx.ffmpeg.run(muxArgs(codec, picture, sound, temp), { signal: ctx.signal }),
    );
  } else await writeViaTemp(out, (temp) => renderTo(job, temp, { muted: false }));
  const { faststart } = CODECS[p.codec];
  if (faststart || chapters.metadata) {
    await writeViaTemp(out, (temp) =>
      ctx.ffmpeg.run(finishArgs(out, temp, { faststart, chapters: chapters.metadata }), {
        signal: ctx.signal,
      }),
    );
  }
  const artifacts: Artifact[] = [
    {
      kind: kindOfCodec(p.codec),
      path: toPosixRel(ctx.projectDir, out),
      label: variant,
    },
  ];
  if (chapters.listed) {
    const { dir, name } = path.parse(out);
    const list = path.join(dir, `${name}.youtube-chapters.txt`);
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
  const { variants } = p;
  const names = videoFileNames(exportBaseName(p.fileName), p.codec, variants);
  const artifacts = await withRenderServer(ctx, async ({ serveUrl, projectBase }) => {
    const chapters = await chaptersFor(ctx, spec, p);
    const made: Artifact[] = [];
    for (const [i, variant] of variants.entries()) {
      const size = variantSize(variant, ctx.workflow.settings);
      const out = path.join(ctx.outDir, names[i]);
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
  const exported = firstArtifact(artifacts).path;
  const probe = await ctx.ffmpeg.probe(path.join(ctx.projectDir, exported));
  const media: MediaValue = {
    type: "media",
    src: projectRef(exported),
    kind: kindOfCodec(p.codec),
    durationMs: framesToMs(spec.durationInFrames, spec.fps),
    hasAudio: !!probe.audio,
  };
  return { outputs: { media }, artifacts };
});

export const stillOutputExecutor = defineExecutor(stillOutputNode, async (ctx, _inputs, p) => {
  const spec = specFor(ctx);
  const names = stillFileNames(exportBaseName(p.fileName), p.presets, p.format);
  const artifacts = await withRenderServer(ctx, async ({ serveUrl, projectBase }) => {
    const made: Artifact[] = [];
    for (const [i, preset] of p.presets.entries()) {
      const size = STILL_PRESETS[preset];
      const out = path.join(ctx.outDir, names[i]);
      ctx.progress(lerp(...RENDER_STAGE, i / p.presets.length), `Still ${preset}`);
      await renderFrame(
        {
          ...renderTarget(
            ctx.env,
            serveUrl,
            { ...spec, ...size, reframe: "follow", assets: { projectBase } },
            p.gl,
          ),
          frame: msToFrames(p.atMs, spec.fps, 0),
          output: out,
          format: p.format,
        },
        { signal: ctx.signal },
      );
      made.push({ kind: "image", path: toPosixRel(ctx.projectDir, out), label: preset });
    }
    return made;
  });
  return {
    outputs: { image: { type: "image", src: projectRef(firstArtifact(artifacts).path) } },
    artifacts,
  };
});
