import { existsSync } from "node:fs";
import { cp, mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { z } from "zod";
import { readJson, writeJson } from "../../engine/json";
import { toPosixRel, toProjectRef } from "../../engine/project-paths";
import { isProjectRef, projectRelPath, STATIC_PROJECT_BASE } from "../../shared/asset-ref";
import { walkItems } from "../../shared/compile/clip";
import { exportProjectNode } from "../../shared/nodes/output";
import { safeName } from "../../shared/safe-name";
import type { TimelineSpec } from "../../shared/schema/timeline";
import { stringsIn } from "../../shared/vars";
import { defineExecutor, type ExecutorContext } from "../engine/context";
import {
  SOURCE_ALIAS,
  SOURCE_DIR,
  WEBPACK_OVERRIDE_DIR,
  WEBPACK_OVERRIDE_FILES,
} from "../render/source-alias";
import { mixed, specFor, type Stage } from "./film-spec";
import { exportSourceRoot, exportUtilsFile, publicDir } from "./util";

const EXPORT_MIX_STAGE: Stage = [0, 0.3];

const EXPORT_PEERS = ["react", "react-dom"];

const EXPORT_DEV_DEPENDENCIES = [
  "@remotion/cli",
  "@remotion/tailwind-v4",
  "tailwindcss",
  "typescript",
  "@types/react",
  "@types/node",
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
import { timelineSpecSchema } from "./shared/schema/timeline";

const spec = timelineSpecSchema.parse(timeline);

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

const CANT_EXPORT_CODE = "This copy of Opsis can't export code.";

const REMOTION_CONFIG_TS = `import { Config } from "@remotion/cli/config";
import { makeWebpackOverride } from "./${[SOURCE_DIR, ...WEBPACK_OVERRIDE_DIR, "webpack-override"].join("/")}";

Config.overrideWebpackConfig(makeWebpackOverride(process.cwd()));
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
    paths: { [`${SOURCE_ALIAS}/*`]: [`./${SOURCE_DIR}/*`] },
  },
  include: ["src"],
};

const readme = (name: string) => `# ${name}

Exported from Opsis. Edit \`src/timeline.json\` or the components in \`src/video\`, then:

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
    utils: exportUtilsFile(ctx.env),
    public: publicDir(ctx.env),
    webpack: path.join(root, ...WEBPACK_OVERRIDE_DIR),
  };
}

async function copySources(ctx: ExecutorContext, dir: string) {
  const src = sourceDirs(ctx);
  if (!existsSync(src.video)) throw new Error(CANT_EXPORT_CODE);
  await mkdir(path.join(dir, "src", "renderer", "lib"), { recursive: true });
  await cp(src.video, path.join(dir, "src", "video"), { recursive: true });
  await cp(src.shared, path.join(dir, "src", "shared"), { recursive: true });
  await cp(src.utils, path.join(dir, "src", "renderer", "lib", "utils.ts"));
  await cp(src.public, path.join(dir, "public"), { recursive: true });
  for (const file of WEBPACK_OVERRIDE_FILES)
    await cp(
      path.join(src.webpack, file),
      path.join(dir, SOURCE_DIR, ...WEBPACK_OVERRIDE_DIR, file),
    );
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

function projectFiles(spec: TimelineSpec): Set<string> {
  const values: unknown[] = [spec.audio.map((a) => a.src), spec.mix?.src];
  walkItems(spec.tracks, (item) => values.push(item.props));
  return new Set(stringsIn(values, isProjectRef).flatMap((ref) => projectRelPath(ref) ?? []));
}

async function copyProjectFiles(ctx: ExecutorContext, spec: TimelineSpec, dir: string) {
  for (const rel of projectFiles(spec)) {
    const target = path.join(dir, "public", "project", rel);
    await mkdir(path.dirname(target), { recursive: true });
    await cp(path.join(ctx.projectDir, rel), target);
  }
}

async function packageJsonFor(ctx: ExecutorContext, name: string, dir: string) {
  const appPkg = await readJson(path.join(ctx.env.appRoot, "package.json"), appPackageSchema);
  if (!appPkg) throw new Error(CANT_EXPORT_CODE);
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

export const exportProjectExecutor = defineExecutor(exportProjectNode, async (ctx, _inputs, p) => {
  const spec = await mixed(ctx, specFor(ctx), EXPORT_MIX_STAGE);
  const name = safeName(p.folderName, { fallback: "output" });
  const dir = path.join(ctx.outDir, name);
  await copySources(ctx, dir);
  await copyProjectFiles(ctx, spec, dir);
  await writeJson(path.join(dir, "package.json"), await packageJsonFor(ctx, name, dir));
  await writeJson(path.join(dir, "src", "timeline.json"), {
    ...spec,
    assets: { projectBase: STATIC_PROJECT_BASE },
  });
  await writeFile(path.join(dir, "src", "index.ts"), INDEX_TS);
  await writeFile(path.join(dir, "src", "export-root.tsx"), EXPORT_ROOT_TSX);
  await writeFile(path.join(dir, "remotion.config.ts"), REMOTION_CONFIG_TS);
  await writeJson(path.join(dir, "tsconfig.json"), TSCONFIG);
  await writeFile(path.join(dir, "README.md"), readme(p.folderName));
  return {
    outputs: { file: { type: "file", src: toProjectRef(ctx.projectDir, dir) } },
    artifacts: [
      { kind: "file", path: toPosixRel(ctx.projectDir, dir), label: "Video project code" },
    ],
  };
});
