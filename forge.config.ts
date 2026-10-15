import { cp, mkdir, rm } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import { MakerDeb } from "@electron-forge/maker-deb";
import { MakerRpm } from "@electron-forge/maker-rpm";
import { MakerSquirrel } from "@electron-forge/maker-squirrel";
import { MakerZIP } from "@electron-forge/maker-zip";
import { FusesPlugin } from "@electron-forge/plugin-fuses";
import { VitePlugin } from "@electron-forge/plugin-vite";
import type { ForgeConfig } from "@electron-forge/shared-types";
import { FuseV1Options, FuseVersion } from "@electron/fuses";
import { installFfmpeg, installHeadlessShell } from "./src/engine/install";
import { engineStatus } from "./src/engine/paths";
import { getCompositorPackage } from "./src/worker/render/compositor-package";
import { bundleRemotionProject, getPrebuiltRemotionBundlePath } from "./src/worker/render/bundle";

const require = createRequire(import.meta.url);
const ROOT = import.meta.dirname;
const ENGINE_DIR = path.join(ROOT, "resources", "engine");
const EXPORT_SRC_DIR = path.join(ROOT, "resources", "export-src");
const LEGAL_FILES = [path.join(ROOT, "LICENSE"), path.join(ROOT, "THIRD_PARTY_NOTICES.md")];
const EDITION = process.env.DEMOFLOW_EDITION === "lite" ? "lite" : "full";

const DARWIN_COMPOSITOR_PACKAGES = [
  "@remotion/compositor-darwin-x64",
  "@remotion/compositor-darwin-arm64",
];
const DARWIN_UNIVERSAL_ARCH_FILES =
  "Contents/Resources/app.asar.unpacked/node_modules/@remotion/compositor-darwin-*/**";

const STAGED_RUNTIME_PACKAGES = ["playwright-core"];

function packageDir(name: string, allowMissing = false): string | null {
  try {
    return path.dirname(require.resolve(`${name}/package.json`, { paths: [ROOT] }));
  } catch (error) {
    if (allowMissing) return null;
    throw error;
  }
}

async function stagePackage(name: string, buildPath: string, allowMissing = false): Promise<void> {
  const source = packageDir(name, allowMissing);
  if (!source) return;
  const destination = path.join(buildPath, "node_modules", name);
  await mkdir(path.dirname(destination), { recursive: true });
  await cp(source, destination, { recursive: true });
}

async function stageCompositorPackages(
  buildPath: string,
  platform: string,
  arch: string,
): Promise<void> {
  const packages =
    platform === "darwin" ? DARWIN_COMPOSITOR_PACKAGES : [getCompositorPackage({ arch, platform })];
  for (const pkg of packages) {
    const allowMissing =
      platform === "darwin" &&
      arch !== "universal" &&
      pkg !== getCompositorPackage({ arch, platform });
    await stagePackage(pkg, buildPath, allowMissing);
  }
}

async function stageEngine(platform: string, arch: string): Promise<void> {
  if (platform !== process.platform || arch !== process.arch)
    throw new Error(
      `The engine is staged for the host only; package ${platform}/${arch} on a matching machine.`,
    );
  if (engineStatus(ENGINE_DIR).complete) return;
  await rm(ENGINE_DIR, { recursive: true, force: true });
  const log = (part: string, progress: number | null, message: string) =>
    process.stdout.write(
      `\r[engine] ${part} ${progress == null ? "" : `${Math.round(progress * 100)}%`} ${message}   `,
    );
  await installFfmpeg(ENGINE_DIR, log);
  await installHeadlessShell(ENGINE_DIR, log);
  process.stdout.write("\n");
}

async function stageExportSource(): Promise<void> {
  await rm(EXPORT_SRC_DIR, { recursive: true, force: true });
  await mkdir(EXPORT_SRC_DIR, { recursive: true });
  await cp(path.join(ROOT, "src", "video"), path.join(EXPORT_SRC_DIR, "video"), {
    recursive: true,
  });
  await cp(path.join(ROOT, "src", "shared"), path.join(EXPORT_SRC_DIR, "shared"), {
    recursive: true,
  });
  await cp(
    path.join(ROOT, "src", "renderer", "lib", "utils.ts"),
    path.join(EXPORT_SRC_DIR, "utils.ts"),
  );
  await cp(path.join(ROOT, "public"), path.join(EXPORT_SRC_DIR, "public"), { recursive: true });
}

const config: ForgeConfig = {
  packagerConfig: {
    name: "Demoflow",
    executableName: "demoflow",
    asar: {
      unpackDir: "node_modules/@remotion/compositor-*",
    },
    osxUniversal: {
      x64ArchFiles: DARWIN_UNIVERSAL_ARCH_FILES,
    },
    extraResource: [...(EDITION === "full" ? [ENGINE_DIR] : []), EXPORT_SRC_DIR, ...LEGAL_FILES],
  },
  rebuildConfig: {},
  makers: [new MakerSquirrel({}), new MakerZIP({}, ["darwin"]), new MakerRpm({}), new MakerDeb({})],
  hooks: {
    prePackage: async (_forgeConfig, platform, arch) => {
      if (EDITION === "full") await stageEngine(platform, arch);
      await stageExportSource();
    },
    packageAfterCopy: async (_forgeConfig, buildPath, _electronVersion, platform, arch) => {
      await bundleRemotionProject({
        projectRoot: ROOT,
        outDir: getPrebuiltRemotionBundlePath(buildPath),
      });
      await stageCompositorPackages(buildPath, platform, arch);
      for (const pkg of STAGED_RUNTIME_PACKAGES) await stagePackage(pkg, buildPath);
    },
  },
  plugins: [
    new VitePlugin({
      build: [
        { entry: "src/main/main.ts", config: "vite.main.config.ts", target: "main" },
        { entry: "src/preload/preload.ts", config: "vite.preload.config.ts", target: "preload" },
        { entry: "src/worker/worker.ts", config: "vite.worker.config.ts", target: "main" },
      ],
      renderer: [{ name: "main_window", config: "vite.renderer.config.ts" }],
    }),
    new FusesPlugin({
      version: FuseVersion.V1,
      [FuseV1Options.RunAsNode]: false,
      [FuseV1Options.EnableCookieEncryption]: true,
      [FuseV1Options.EnableNodeOptionsEnvironmentVariable]: false,
      [FuseV1Options.EnableNodeCliInspectArguments]: false,
      [FuseV1Options.EnableEmbeddedAsarIntegrityValidation]: true,
      [FuseV1Options.OnlyLoadAppFromAsar]: true,
    }),
  ],
};

export default config;
