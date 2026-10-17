import { existsSync } from "node:fs";
import { cp, mkdir, readFile, rename, utimes, writeFile } from "node:fs/promises";
import path from "node:path";
import type { z } from "zod";
import { mapItems } from "../../shared/compile/clip";
import { PROJECT_NAME_VAR } from "../../shared/compile/compile";
import {
  exportProjectNode,
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
import { CODECS, hasSoundTrack } from "../render/codecs";
import { getServeUrl, renderFrame, renderTimeline } from "../render/render";
import { finishArgs, muxArgs, pictureKey, prunePictures } from "../render/picture";
import {
  chapterList,
  chapterMetadata,
  filmChapters,
  MIN_LISTED_CHAPTERS,
} from "../../shared/timeline/chapters";
import { publicDir, refToPath } from "./util";
import { buildMixPlan, hasSound } from "../../shared/mix/plan";
import { applyMixLevels, withMasterStem } from "../../shared/mix/levels";
import { renderMix } from "../mix/render";
import { projectPaths, toPosixRel, toProjectRef } from "../../engine/project-paths";
import { framesToMs, msToFrames } from "../../shared/timeline/time";
import { slugify } from "../../shared/project-folder";
import { isProjectRef, projectRef, projectRelPath } from "../../shared/asset-ref";

type RenderParams = z.infer<typeof renderNode.params>;

const relativePath = (ctx: ExecutorContext, file: string) => toPosixRel(ctx.projectDir, file);

function firstArtifact(artifacts: Artifact[]): Artifact {
  const [first] = artifacts;
  if (!first) throw new Error("Nothing was exported. Pick at least one size.");
  return first;
}

function specFor(ctx: ExecutorContext): TimelineSpec {
  const result = ctx.compile();
  const error = result.errors[ctx.node.id];
  if (error) throw new Error(error);
  const spec = result.specs[ctx.node.id];
  if (!spec) throw new Error("Connect a clip to this node.");
  return spec;
}

async function mixed(ctx: ExecutorContext, spec: TimelineSpec, from: number, to: number) {
  const plan = buildMixPlan(spec, ctx.workflow.mix);
  if (!hasSound(plan)) return applyMixLevels(spec, ctx.workflow.mix);
  const result = await renderMix(plan, {
    projectDir: ctx.projectDir,
    resolve: (ref) => refToPath(ctx, ref),
    ffmpeg: ctx.ffmpeg,
    signal: ctx.signal,
    onProgress: (p, message) => ctx.progress(from + (to - from) * p, message),
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
    ctx.progress(0.1 + p * 0.1, "Bundling compositions"),
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
  const dir = path.join(projectPaths(ctx.projectDir).cache, "pictures");
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
  chapters: ReturnType<typeof filmChapters>;
  metadata: string | null;
  listed: boolean;
};

async function chaptersFor(ctx: ExecutorContext, spec: TimelineSpec, p: RenderParams) {
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
  return { chapters, metadata, listed } satisfies Chapters;
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
  const faststart = p.faststart && CODECS[p.codec].faststart;
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
  const spec = await mixed(ctx, specFor(ctx), 0, 0.05);
  const { serveUrl, server } = await prepare(ctx);
  const artifacts: Artifact[] = [];
  const base = slugify(p.fileName, "output");
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
          ctx.progress(0.2 + 0.8 * ((i + x) / variants.length), `Rendering ${variant}`),
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
  const base = slugify(ctx.vars[PROJECT_NAME_VAR], "output");
  try {
    for (const [i, preset] of p.presets.entries()) {
      const size = STILL_PRESETS[preset];
      const out = path.join(ctx.outDir, `${base}-${preset}.${p.format === "jpeg" ? "jpg" : "png"}`);
      ctx.progress(0.2 + 0.8 * (i / p.presets.length), `Still ${preset}`);
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

const EXPORT_DEPENDENCIES = [
  "react",
  "react-dom",
  "zod",
  "culori",
  "cn",
  "opentype.js",
  "@paper-design/shaders-react",
  "@fontsource-variable/geist",
  "@fontsource/inter",
  "@fontsource/manrope",
  "date-fns",
];

const EXPORT_DEV_DEPENDENCIES = [
  "@remotion/cli",
  "@remotion/tailwind-v4",
  "tailwindcss",
  "typescript",
  "@types/react",
];

const exportedDependency = (name: string) =>
  name === "remotion" || name.startsWith("@remotion/") || EXPORT_DEPENDENCIES.includes(name);

const INDEX_TS = `import { registerRoot } from "remotion";
import "./video/video.css";
import { ExportRoot } from "./export-root";

registerRoot(ExportRoot);
`;

const EXPORT_ROOT_TSX = `import { Composition } from "remotion";
import { DemoTimeline } from "./video/demo-timeline";
import timeline from "./timeline.json";
import type { TimelineSpec } from "./shared/schema/timeline";

const spec = timeline as TimelineSpec;

export const ExportRoot = () => (
  <Composition
    id="DemoTimeline"
    component={DemoTimeline}
    defaultProps={spec}
    durationInFrames={spec.durationInFrames}
    fps={spec.fps}
    width={spec.width}
    height={spec.height}
  />
);
`;

const REMOTION_CONFIG_TS = `import path from "node:path";
import { Config } from "@remotion/cli/config";
import { enableTailwind } from "@remotion/tailwind-v4";

Config.overrideWebpackConfig((c) => {
  const t = enableTailwind(c);
  return { ...t, resolve: { ...t.resolve, alias: { ...(t.resolve?.alias as object), "@": path.join(process.cwd(), "src") } } };
});
`;

const TSCONFIG = {
  compilerOptions: {
    target: "ESNext",
    module: "preserve",
    moduleResolution: "bundler",
    jsx: "react-jsx",
    strict: true,
    skipLibCheck: true,
    resolveJsonModule: true,
    noEmit: true,
    paths: { "@/*": ["./src/*"] },
  },
  include: ["src"],
};

const readme = (name: string) => `# ${name}

Exported from Demoflow. Edit \`src/timeline.json\` or the components in \`src/video\`, then:

\`\`\`sh
bun install
bun run studio
\`\`\`
`;

function sourceDirs(ctx: ExecutorContext) {
  const root = ctx.env.isPackaged
    ? path.join(ctx.env.resourcesPath, "export-src")
    : path.join(ctx.env.appRoot, "src");
  return {
    video: path.join(root, "video"),
    shared: path.join(root, "shared"),
    utils: ctx.env.isPackaged
      ? path.join(root, "utils.ts")
      : path.join(root, "renderer", "lib", "utils.ts"),
    public: publicDir(ctx.env),
  };
}

async function copySources(ctx: ExecutorContext, dir: string) {
  const src = sourceDirs(ctx);
  if (!existsSync(src.video)) throw new Error("This copy of Demoflow can't export code.");
  await mkdir(path.join(dir, "src", "renderer", "lib"), { recursive: true });
  await cp(src.video, path.join(dir, "src", "video"), { recursive: true });
  await cp(src.shared, path.join(dir, "src", "shared"), { recursive: true });
  await cp(src.utils, path.join(dir, "src", "renderer", "lib", "utils.ts"));
  await cp(src.public, path.join(dir, "public"), { recursive: true });
}

function projectRefs(spec: TimelineSpec): Set<string> {
  const refs = new Set<string>();
  const collect = (v: unknown) => {
    if (typeof v === "string" && isProjectRef(v)) refs.add(v);
    else if (Array.isArray(v)) v.forEach(collect);
    else if (v && typeof v === "object") Object.values(v).forEach(collect);
  };
  mapItems(spec.tracks, (item) => {
    collect(item.props);
    return item;
  });
  spec.audio.forEach((a) => collect(a.src));
  if (spec.mix) collect(spec.mix.src);
  return refs;
}

async function copyProjectFiles(ctx: ExecutorContext, spec: TimelineSpec, dir: string) {
  for (const ref of projectRefs(spec)) {
    const target = path.join(dir, "public", "project", projectRelPath(ref));
    await mkdir(path.dirname(target), { recursive: true });
    await cp(refToPath(ctx, ref), target);
  }
}

async function packageJsonFor(ctx: ExecutorContext, name: string) {
  const appPkg = JSON.parse(await readFile(path.join(ctx.env.appRoot, "package.json"), "utf8")) as {
    dependencies: Record<string, string>;
    devDependencies?: Record<string, string>;
  };
  const versions: Partial<Record<string, string>> = {
    ...appPkg.devDependencies,
    ...appPkg.dependencies,
  };
  const pick = (names: string[]) =>
    Object.fromEntries(names.flatMap((n) => (versions[n] ? [[n, versions[n]]] : [])));
  return {
    name,
    private: true,
    type: "module",
    scripts: {
      studio: "remotion studio src/index.ts",
      render: "remotion render src/index.ts DemoTimeline out/video.mp4",
    },
    dependencies: pick(Object.keys(appPkg.dependencies).filter(exportedDependency)),
    devDependencies: pick(EXPORT_DEV_DEPENDENCIES),
  };
}

const json = (value: unknown) => JSON.stringify(value, null, 2);

export const exportProjectExecutor = defineExecutor(exportProjectNode, async (ctx, _inputs, p) => {
  const spec = await mixed(ctx, specFor(ctx), 0, 0.3);
  const name = slugify(p.folderName, "output");
  const dir = path.join(ctx.outDir, name);
  await copySources(ctx, dir);
  await copyProjectFiles(ctx, spec, dir);
  await writeFile(path.join(dir, "package.json"), json(await packageJsonFor(ctx, name)));
  await writeFile(
    path.join(dir, "src", "timeline.json"),
    json({ ...spec, assets: { projectBase: "static:" } }),
  );
  await writeFile(path.join(dir, "src", "index.ts"), INDEX_TS);
  await writeFile(path.join(dir, "src", "export-root.tsx"), EXPORT_ROOT_TSX);
  await writeFile(path.join(dir, "remotion.config.ts"), REMOTION_CONFIG_TS);
  await writeFile(path.join(dir, "tsconfig.json"), json(TSCONFIG));
  await writeFile(path.join(dir, "README.md"), readme(p.folderName));
  return {
    outputs: { file: { type: "file", src: toProjectRef(ctx.projectDir, dir) } },
    artifacts: [{ kind: "file", path: relativePath(ctx, dir), label: "Video project code" }],
  };
});
