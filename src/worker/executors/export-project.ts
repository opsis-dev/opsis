import { existsSync } from "node:fs";
import { cp, mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { z } from "zod";
import { walkItems } from "../../shared/compile/clip";
import { exportProjectNode } from "../../shared/nodes/output";
import type { TimelineSpec } from "../../shared/schema/timeline";
import { slugify } from "../../shared/project-folder";
import { isProjectRef, projectRelPath } from "../../shared/asset-ref";
import { readJson } from "../../engine/json";
import { toPosixRel, toProjectRef } from "../../engine/project-paths";
import { defineExecutor, type ExecutorContext } from "../engine/context";
import { mixed, specFor, type Stage } from "./output";
import { exportSourceRoot, publicDir, refToPath } from "./util";

const MIX_STAGE: Stage = [0, 0.3];

const EXPORT_PEERS = ["react", "react-dom"];

const EXPORT_DEV_DEPENDENCIES = [
  "@remotion/cli",
  "@remotion/tailwind-v4",
  "tailwindcss",
  "typescript",
  "@types/react",
];

const SOURCE_FILE = /\.(ts|tsx|mts|css)$/;

const IMPORT_SPECIFIER = /(?:\bfrom\s*|\bimport\s*\(?\s*|@import\s+)["']([^"'\n]+)["']/g;

const appPackageSchema = z.object({
  dependencies: z.record(z.string(), z.string()),
  devDependencies: z.record(z.string(), z.string()).optional(),
});

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
  const alias = t.resolve?.alias;
  return {
    ...t,
    resolve: {
      ...t.resolve,
      alias: { ...(Array.isArray(alias) ? {} : alias), "@": path.join(process.cwd(), "src") },
    },
  };
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

const isRemotion = (name: string) => name === "remotion" || name.startsWith("@remotion/");

function packageOf(specifier: string): string | null {
  if (/^(\.|\/|@\/|node:)/.test(specifier)) return null;
  const parts = specifier.split("/");
  return specifier.startsWith("@") ? parts.slice(0, 2).join("/") : parts[0];
}

function sourceDirs(ctx: ExecutorContext) {
  const root = exportSourceRoot(ctx.env);
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

async function importedPackages(srcDir: string): Promise<Set<string>> {
  const files = (await readdir(srcDir, { recursive: true })).filter((f) => SOURCE_FILE.test(f));
  const names = new Set<string>();
  for (const file of files) {
    const text = await readFile(path.join(srcDir, file), "utf8");
    for (const [, specifier] of text.matchAll(IMPORT_SPECIFIER)) {
      const name = packageOf(specifier);
      if (name) names.add(name);
    }
  }
  return names;
}

function projectRefs(spec: TimelineSpec): Set<string> {
  const refs = new Set<string>();
  const collect = (v: unknown) => {
    if (typeof v === "string" && isProjectRef(v)) refs.add(v);
    else if (Array.isArray(v)) v.forEach(collect);
    else if (v && typeof v === "object") Object.values(v).forEach(collect);
  };
  walkItems(spec.tracks, (item) => collect(item.props));
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

async function packageJsonFor(ctx: ExecutorContext, name: string, dir: string) {
  const appPkg = await readJson(path.join(ctx.env.appRoot, "package.json"), appPackageSchema);
  if (!appPkg) throw new Error("This copy of Demoflow can't export code.");
  const versions: Partial<Record<string, string>> = {
    ...appPkg.devDependencies,
    ...appPkg.dependencies,
  };
  const pick = (names: string[]) =>
    Object.fromEntries(names.flatMap((n) => (versions[n] ? [[n, versions[n]]] : [])));
  const imported = await importedPackages(path.join(dir, "src"));
  const runtime = Object.keys(appPkg.dependencies).filter(
    (n) =>
      isRemotion(n) ||
      EXPORT_PEERS.includes(n) ||
      (imported.has(n) && !EXPORT_DEV_DEPENDENCIES.includes(n)),
  );
  return {
    name,
    private: true,
    type: "module",
    scripts: {
      studio: "remotion studio src/index.ts",
      render: "remotion render src/index.ts DemoTimeline out/video.mp4",
    },
    dependencies: pick(runtime),
    devDependencies: pick(EXPORT_DEV_DEPENDENCIES),
  };
}

const json = (value: unknown) => JSON.stringify(value, null, 2);

export const exportProjectExecutor = defineExecutor(exportProjectNode, async (ctx, _inputs, p) => {
  const spec = await mixed(ctx, specFor(ctx), MIX_STAGE);
  const name = slugify(p.folderName, "output");
  const dir = path.join(ctx.outDir, name);
  await copySources(ctx, dir);
  await copyProjectFiles(ctx, spec, dir);
  await writeFile(path.join(dir, "package.json"), json(await packageJsonFor(ctx, name, dir)));
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
    artifacts: [
      { kind: "file", path: toPosixRel(ctx.projectDir, dir), label: "Video project code" },
    ],
  };
});
