import { chmod, copyFile, cp, mkdir, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { MakerDeb } from "@electron-forge/maker-deb";
import { MakerRpm } from "@electron-forge/maker-rpm";
import { MakerSquirrel } from "@electron-forge/maker-squirrel";
import { MakerZIP } from "@electron-forge/maker-zip";
import { MakerAppImage } from "@reforged/maker-appimage";
import { FusesPlugin } from "@electron-forge/plugin-fuses";
import { VitePlugin } from "@electron-forge/plugin-vite";
import type { ForgeConfig } from "@electron-forge/shared-types";
import { FuseV1Options, FuseVersion } from "@electron/fuses";
import { installFfmpeg, installHeadlessShell } from "./src/engine/install";
import { writeChromiumCredits } from "./src/engine/chromium-credits";
import {
  CHROMIUM_CREDITS_FILE,
  ENGINE_LICENCES_FOLDER,
  enginePaths,
  engineStatus,
} from "./src/engine/paths";
import { getCompositorPackage } from "./src/worker/render/compositor-package";
import { bundleRemotionProject, getPrebuiltRemotionBundlePath } from "./src/worker/render/bundle";

const require = createRequire(import.meta.url);
const ROOT = import.meta.dirname;
const ENGINE_DIR = path.join(ROOT, "resources", "engine");
const CHROMIUM_CREDITS = path.join(ENGINE_DIR, "LICENSES.chromium.html");
const ENGINE_LICENCES_DIR = path.join(ROOT, "resources", ENGINE_LICENCES_FOLDER);
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
  if (engineStatus(ENGINE_DIR).complete && existsSync(CHROMIUM_CREDITS)) return;
  await rm(ENGINE_DIR, { recursive: true, force: true });
  const log = (part: string, progress: number | null, message: string) =>
    process.stdout.write(
      `\r[engine] ${part} ${progress == null ? "" : `${Math.round(progress * 100)}%`} ${message}   `,
    );
  await installFfmpeg(ENGINE_DIR, log);
  await installHeadlessShell(ENGINE_DIR, log);
  process.stdout.write("\n[engine] Saving the recording browser's licences\n");
  const credits = await writeChromiumCredits(
    enginePaths(ENGINE_DIR).headlessShell,
    CHROMIUM_CREDITS,
    path.join(os.homedir(), ".cache", "demoflow-chromium-credits"),
  );
  process.stdout.write(
    `[engine] Chromium ${credits.version}: ${credits.bytes} bytes of licences\n`,
  );
}

async function stageEngineLicences(): Promise<void> {
  await rm(ENGINE_LICENCES_DIR, { recursive: true, force: true });
  await mkdir(ENGINE_LICENCES_DIR, { recursive: true });
  await copyFile(CHROMIUM_CREDITS, path.join(ENGINE_LICENCES_DIR, CHROMIUM_CREDITS_FILE));
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

const LINUX_CATEGORIES = ["AudioVideo", "Video"] as const;
const APPIMAGE_LAUNCHER = "demoflow-launcher";
const ICONS_DIR = path.join(ROOT, "brand", "icons");
const icon = (size: number) => path.join(ICONS_DIR, `${size}.png`);

const config: ForgeConfig = {
  outDir: process.env.DEMOFLOW_OUT_DIR || "out",
  packagerConfig: {
    name: "Demoflow",
    executableName: "demoflow",
    asar: {
      unpackDir: "node_modules/@remotion/compositor-*",
    },
    osxUniversal: {
      x64ArchFiles: DARWIN_UNIVERSAL_ARCH_FILES,
    },
    extraResource: [
      EDITION === "full" ? ENGINE_DIR : ENGINE_LICENCES_DIR,
      EXPORT_SRC_DIR,
      ICONS_DIR,
      ...LEGAL_FILES,
    ],
  },
  rebuildConfig: {},
  makers: [
    new MakerSquirrel({}),
    new MakerZIP({}, ["darwin"]),
    new MakerRpm({ options: { categories: [...LINUX_CATEGORIES] } }),
    new MakerDeb({
      options: { categories: [...LINUX_CATEGORIES], section: "video", icon: icon(512) },
    }),
    new MakerAppImage({
      options: {
        categories: [...LINUX_CATEGORIES],
        bin: APPIMAGE_LAUNCHER,
        icon: {
          scalable: path.join(ROOT, "brand", "logo.svg"),
          "512x512": icon(512),
          "256x256": icon(256),
          "128x128": icon(128),
          "64x64": icon(64),
          "48x48": icon(48),
          "32x32": icon(32),
          "16x16": icon(16),
          default: "256x256",
        },
      },
    }),
  ],
  hooks: {
    prePackage: async (_forgeConfig, platform, arch) => {
      await stageEngine(platform, arch);
      if (EDITION === "lite") await stageEngineLicences();
      await stageExportSource();
    },
    postPackage: async (_forgeConfig, { platform, outputPaths }) => {
      if (platform !== "linux") return;
      for (const out of outputPaths) {
        const launcher = path.join(out, APPIMAGE_LAUNCHER);
        await copyFile(path.join(ROOT, "scripts", "appimage-launcher.sh"), launcher);
        await chmod(launcher, 0o755);
      }
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
