import { existsSync } from "node:fs";
import { cp, mkdir, readFile, rename, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Codec } from "@remotion/renderer";
import { mapItems, msToFrames } from "../../shared/compile/clip";
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
import { CODEC_EXT, getServeUrl, renderFrame, renderTimeline } from "../render/render";
import { refToPath, slug } from "./util";

function specFor(ctx: ExecutorContext): TimelineSpec {
  const result = ctx.compile();
  const error = result.errors[ctx.node.id];
  if (error) throw new Error(error);
  const spec = result.specs[ctx.node.id];
  if (!spec) throw new Error("Connect a clip to this node.");
  return spec;
}

async function prepare(ctx: ExecutorContext) {
  ctx.progress(0, "Preparing render engine");
  const serveUrl = await getServeUrl(ctx.env, (p) =>
    ctx.progress(0.1 + p * 0.1, "Bundling compositions"),
  );
  const server = await startAssetServer(ctx.projectDir);
  return { serveUrl, server };
}

export const renderExecutor = defineExecutor(renderNode, async (ctx, _inputs, p) => {
  const spec = specFor(ctx);
  const { serveUrl, server } = await prepare(ctx);
  const artifacts: Artifact[] = [];
  const ext = CODEC_EXT[p.codec];
  const base = slug(p.fileName);
  try {
    const variants = p.variants as Variant[];
    for (const [i, variant] of variants.entries()) {
      const size = variantSize(variant, ctx.workflow.settings);
      const out = path.join(
        ctx.outDir,
        `${base}${variants.length > 1 ? `-${variant.replace(":", "x")}` : ""}.${ext}`,
      );
      const variantSpec: TimelineSpec = {
        ...spec,
        ...size,
        reframe: p.reframe,
        assets: { projectBase: server.url },
      };
      ctx.log(
        "info",
        `Exporting ${variant} (${size.width}×${size.height}) as ${path.basename(out)}`,
      );
      await renderTimeline({
        env: ctx.env,
        serveUrl,
        spec: variantSpec,
        output: out,
        codec: p.codec as Codec,
        crf: p.crf,
        scale: p.scale,
        gl: p.gl,
        signal: ctx.signal,
        onProgress: (x) =>
          ctx.progress(0.2 + 0.8 * ((i + x) / variants.length), `Rendering ${variant}`),
      });
      if (p.faststart && (p.codec === "h264" || p.codec === "h265")) {
        const tmp = `${out}.faststart${path.extname(out)}`;
        await ctx.ffmpeg.run(["-i", out, "-c", "copy", "-movflags", "+faststart", tmp], {
          signal: ctx.signal,
        });
        await rename(tmp, out);
      }
      artifacts.push({
        kind: p.codec === "gif" ? "image" : "video",
        path: path.relative(ctx.projectDir, out).split(path.sep).join("/"),
        label: variant,
      });
    }
  } finally {
    await server.close();
  }
  const first = artifacts[0];
  const media: MediaValue = {
    type: "media",
    src: `project:${first.path}`,
    kind: p.codec === "gif" ? "image" : "video",
    durationMs: (spec.durationInFrames / spec.fps) * 1000,
  };
  return { outputs: { media }, artifacts };
});

export const stillOutputExecutor = defineExecutor(stillOutputNode, async (ctx, _inputs, p) => {
  const spec = specFor(ctx);
  const { serveUrl, server } = await prepare(ctx);
  const artifacts: Artifact[] = [];
  try {
    for (const [i, preset] of p.presets.entries()) {
      const size = STILL_PRESETS[preset];
      const out = path.join(
        ctx.outDir,
        `${slug(ctx.vars["Project name"])}-${preset}.${p.format === "jpeg" ? "jpg" : "png"}`,
      );
      ctx.progress(0.2 + 0.8 * (i / p.presets.length), `Still ${preset}`);
      await renderFrame({
        env: ctx.env,
        serveUrl,
        spec: { ...spec, ...size, reframe: "follow", assets: { projectBase: server.url } },
        frame: msToFrames(p.atMs, spec.fps),
        output: out,
        format: p.format,
        gl: p.gl,
        signal: ctx.signal,
      });
      artifacts.push({
        kind: "image",
        path: path.relative(ctx.projectDir, out).split(path.sep).join("/"),
        label: preset,
      });
    }
  } finally {
    await server.close();
  }
  return { outputs: { image: { type: "image", src: `project:${artifacts[0].path}` } }, artifacts };
});

function sourceDirs(ctx: ExecutorContext) {
  if (!ctx.env.isPackaged) {
    const src = path.join(ctx.env.appRoot, "src");
    return {
      video: path.join(src, "video"),
      shared: path.join(src, "shared"),
      utils: path.join(src, "renderer", "lib", "utils.ts"),
      public: path.join(ctx.env.appRoot, "public"),
    };
  }
  const res = path.join(ctx.env.resourcesPath, "export-src");
  return {
    video: path.join(res, "video"),
    shared: path.join(res, "shared"),
    utils: path.join(res, "utils.ts"),
    public: path.join(res, "public"),
  };
}

export const exportProjectExecutor = defineExecutor(exportProjectNode, async (ctx, _inputs, p) => {
  const spec = specFor(ctx);
  const dir = path.join(ctx.outDir, slug(p.folderName));
  const src = sourceDirs(ctx);
  if (!existsSync(src.video)) throw new Error("This copy of Demoflow can't export code.");
  await mkdir(path.join(dir, "src", "renderer", "lib"), { recursive: true });
  await cp(src.video, path.join(dir, "src", "video"), { recursive: true });
  await cp(src.shared, path.join(dir, "src", "shared"), { recursive: true });
  await cp(src.utils, path.join(dir, "src", "renderer", "lib", "utils.ts"));
  await cp(src.public, path.join(dir, "public"), { recursive: true });

  const refs = new Set<string>();
  const collect = (v: unknown) => {
    if (typeof v === "string" && v.startsWith("project:")) refs.add(v);
    else if (Array.isArray(v)) v.forEach(collect);
    else if (v && typeof v === "object") Object.values(v).forEach(collect);
  };
  mapItems(spec.tracks, (item) => {
    collect(item.props);
    return item;
  });
  spec.audio.forEach((a) => collect(a.src));
  for (const ref of refs) {
    const relPath = ref.slice("project:".length);
    const target = path.join(dir, "public", "project", relPath);
    await mkdir(path.dirname(target), { recursive: true });
    await cp(refToPath(ctx, ref), target);
  }

  const appPkg = JSON.parse(await readFile(path.join(ctx.env.appRoot, "package.json"), "utf8")) as {
    dependencies: Record<string, string>;
    devDependencies?: Record<string, string>;
  };
  const pick = (names: string[]) =>
    Object.fromEntries(
      names
        .filter((n) => appPkg.dependencies[n] ?? appPkg.devDependencies?.[n])
        .map((n) => [n, appPkg.dependencies[n] ?? appPkg.devDependencies![n]]),
    );
  const deps = Object.keys(appPkg.dependencies).filter(
    (d) =>
      d === "remotion" ||
      d.startsWith("@remotion/") ||
      [
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
      ].includes(d),
  );
  await writeFile(
    path.join(dir, "package.json"),
    JSON.stringify(
      {
        name: slug(p.folderName),
        private: true,
        type: "module",
        scripts: {
          studio: "remotion studio src/index.ts",
          render: "remotion render src/index.ts DemoTimeline out/video.mp4",
        },
        dependencies: pick(deps),
        devDependencies: pick([
          "@remotion/cli",
          "@remotion/tailwind-v4",
          "tailwindcss",
          "typescript",
          "@types/react",
        ]),
      },
      null,
      2,
    ),
  );
  await writeFile(
    path.join(dir, "src", "timeline.json"),
    JSON.stringify({ ...spec, assets: { projectBase: "static:" } }, null, 2),
  );
  await writeFile(
    path.join(dir, "src", "index.ts"),
    `import { registerRoot } from "remotion";\nimport "./video/video.css";\nimport { ExportRoot } from "./ExportRoot";\n\nregisterRoot(ExportRoot);\n`,
  );
  await writeFile(
    path.join(dir, "src", "ExportRoot.tsx"),
    `import { Composition } from "remotion";\nimport { DemoTimeline } from "./video/DemoTimeline";\nimport timeline from "./timeline.json";\nimport type { TimelineSpec } from "./shared/schema/timeline";\n\nconst spec = timeline as TimelineSpec;\n\nexport const ExportRoot = () => (\n  <Composition id="DemoTimeline" component={DemoTimeline} defaultProps={spec} durationInFrames={spec.durationInFrames} fps={spec.fps} width={spec.width} height={spec.height} />\n);\n`,
  );
  await writeFile(
    path.join(dir, "remotion.config.ts"),
    `import path from "node:path";\nimport { Config } from "@remotion/cli/config";\nimport { enableTailwind } from "@remotion/tailwind-v4";\n\nConfig.overrideWebpackConfig((c) => {\n  const t = enableTailwind(c);\n  return { ...t, resolve: { ...t.resolve, alias: { ...(t.resolve?.alias as object), "@": path.join(process.cwd(), "src") } } };\n});\n`,
  );
  await writeFile(
    path.join(dir, "tsconfig.json"),
    JSON.stringify(
      {
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
      },
      null,
      2,
    ),
  );
  await writeFile(
    path.join(dir, "README.md"),
    `# ${p.folderName}\n\nExported from Demoflow. Edit \`src/timeline.json\` or the components in \`src/video\`, then:\n\n\`\`\`sh\nbun install\nbun run studio\n\`\`\`\n`,
  );
  const relDir = path.relative(ctx.projectDir, dir).split(path.sep).join("/");
  return {
    outputs: { file: { type: "file", src: `project:${relDir}` } },
    artifacts: [{ kind: "file", path: relDir, label: "Remotion project" }],
  };
});
