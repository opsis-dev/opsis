import { existsSync } from "node:fs";
import { cp, mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { isProjectRef, projectRelPath, STATIC_PROJECT_BASE } from "@opsis/core/asset-ref";
import { walkItems } from "@opsis/core/compile/clip";
import { EXPORT_FOLDER_FALLBACK, exportProjectNode } from "@opsis/core/nodes/output";
import { safeName } from "@opsis/core/safe-name";
import type { TimelineSpec } from "@opsis/core/schema/timeline";
import { stringsIn } from "@opsis/core/vars";
import {
  DEPENDENCY_MANIFESTS,
  EXPORTED_PACKAGES,
  EXPORTED_SOURCES,
  packageManifest,
  VIDEO_SOURCE,
} from "@opsis/sidecar/source-layout";
import { z } from "zod";
import { readJson, writeJson } from "../../engine/json";
import { toPosixRel, toProjectRef } from "../../engine/project-paths";
import { defineExecutor, type ExecutorContext } from "../engine/context";
import { mixed, specFor, type Stage } from "./film-spec";
import { exportedSource, publicDir } from "./util";

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

const importsSchema = z.object({ imports: z.record(z.string(), z.string()).optional() });

const PACKAGE_SOURCE_PREFIX = "./src/";

const importsInCopy = (imports: Record<string, string>) =>
  Object.fromEntries(
    Object.entries(imports).map(([key, target]) => [
      key,
      target.startsWith(PACKAGE_SOURCE_PREFIX)
        ? `./${target.slice(PACKAGE_SOURCE_PREFIX.length)}`
        : target,
    ]),
  );

const manifestSchema = z.object({
  dependencies: z.record(z.string(), z.string()).optional(),
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
import { timelineSpecSchema } from "./core/schema/timeline";

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

const EXPORTED_PACKAGE_DIRS = Object.entries(EXPORTED_PACKAGES);

const REMOTION_CONFIG_TS = `import path from "node:path";
import { Config } from "@remotion/cli/config";
import { makeBundlerOverride } from "./src/sidecar/bundler-override";

const src = path.join(process.cwd(), "src");

Config.setRspack(true);
Config.overrideRspackConfig(
  makeBundlerOverride(path.join(src, "video"), {
${EXPORTED_PACKAGE_DIRS.map(([name, dir]) => `    "${name}": path.join(src, "${dir}"),`).join("\n")}
  }),
);
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
    allowImportingTsExtensions: true,
    noEmit: true,
    paths: Object.fromEntries(
      EXPORTED_PACKAGE_DIRS.map(([name, dir]) => [`${name}/*`, [`./src/${dir}/*`]]),
    ),
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
  if (/^(\.|\/|#|@\/|@opsis\/|node:)/.test(specifier)) return null;
  const parts = specifier.split("/");
  return specifier.startsWith("@") ? parts.slice(0, 2).join("/") : parts[0];
}

async function copySources(ctx: ExecutorContext, dir: string) {
  if (!existsSync(exportedSource(ctx.env, { from: VIDEO_SOURCE, to: "video" })))
    throw new Error(CANT_EXPORT_CODE);
  for (const source of EXPORTED_SOURCES)
    await cp(exportedSource(ctx.env, source), path.join(dir, "src", source.to), {
      recursive: true,
    });
  await cp(publicDir(ctx.env), path.join(dir, "public"), { recursive: true });
  for (const folder of Object.values(EXPORTED_PACKAGES)) {
    const manifest = await readJson(
      exportedSource(ctx.env, packageManifest(folder)),
      importsSchema,
    );
    if (manifest?.imports)
      await writeJson(path.join(dir, "src", folder, "package.json"), {
        type: "module",
        imports: importsInCopy(manifest.imports),
      });
  }
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

function checkProjectFiles(ctx: ExecutorContext, spec: TimelineSpec) {
  for (const rel of projectFiles(spec))
    if (!existsSync(path.join(ctx.projectDir, rel)))
      throw new Error(`“${rel}” is missing from the project. Choose the file again, or remove it.`);
}

async function packageJsonFor(ctx: ExecutorContext, name: string, dir: string) {
  const manifests = await Promise.all(
    DEPENDENCY_MANIFESTS.map((source) => readJson(exportedSource(ctx.env, source), manifestSchema)),
  );
  if (manifests.some((manifest) => !manifest)) throw new Error(CANT_EXPORT_CODE);
  const published = (deps: Record<string, string> = {}) =>
    Object.entries(deps).filter(([, version]) => !version.startsWith("workspace:"));
  const dependencies = Object.fromEntries(
    manifests.flatMap((manifest) => published(manifest?.dependencies)),
  );
  const versions: Partial<Record<string, string>> = {
    ...Object.fromEntries(manifests.flatMap((manifest) => published(manifest?.devDependencies))),
    ...dependencies,
  };
  const pick = (names: string[]) =>
    Object.fromEntries(names.flatMap((n) => (versions[n] ? [[n, versions[n]]] : [])));
  const imported = await importedPackages(path.join(dir, "src"));
  const runtime = Object.keys(dependencies).filter(
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
  const name = safeName(p.folderName, { fallback: EXPORT_FOLDER_FALLBACK });
  const dir = path.join(ctx.outDir, name);
  checkProjectFiles(ctx, spec);
  await rm(dir, { recursive: true, force: true });
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
    outputs: { folder: { type: "folder", src: toProjectRef(ctx.projectDir, dir) } },
    artifacts: [
      { kind: "file", path: toPosixRel(ctx.projectDir, dir), label: "Video project code" },
    ],
  };
});
