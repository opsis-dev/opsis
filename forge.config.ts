import { existsSync } from "node:fs";
import { chmod, copyFile, cp, mkdir, rm } from "node:fs/promises";
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
import { MakerAppImage } from "@reforged/maker-appimage";
import { writeChromiumCredits } from "./src/engine/chromium-credits";
import { installFfmpeg, installHeadlessShell, type InstallProgress } from "./src/engine/install";
import { CHROMIUM_CREDITS_FILE, ENGINE_LICENCES_FOLDER, engineStatus } from "./src/engine/paths";
import { percentText } from "./src/shared/format";
import { bundleRemotionProject, getPrebuiltRemotionBundlePath } from "./src/worker/render/bundle";
import { getCompositorPackage } from "./src/worker/render/compositor-package";

const require = createRequire(import.meta.url);
const ROOT = import.meta.dirname;
const ENGINE_DIR = path.join(ROOT, "resources", "engine");
const CHROMIUM_CREDITS = path.join(ENGINE_DIR, CHROMIUM_CREDITS_FILE);
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

async function stagePackage(
  name: string,
  buildPath: string,
  { optional = false } = {},
): Promise<void> {
  let source: string;
  try {
    source = path.dirname(require.resolve(`${name}/package.json`, { paths: [ROOT] }));
  } catch (error) {
    if (optional) return;
    throw error;
  }
  const destination = path.join(buildPath, "node_modules", name);
  await mkdir(path.dirname(destination), { recursive: true });
  await cp(source, destination, { recursive: true });
}

async function stageCompositorPackages(
  buildPath: string,
  platform: string,
  arch: string,
): Promise<void> {
  const universal = platform === "darwin" && arch === "universal";
  const host = universal ? null : getCompositorPackage({ arch, platform });
  const required = host ? [host] : DARWIN_COMPOSITOR_PACKAGES;
  for (const pkg of required) await stagePackage(pkg, buildPath);
  if (platform !== "darwin") return;
  for (const pkg of DARWIN_COMPOSITOR_PACKAGES)
    if (!required.includes(pkg)) await stagePackage(pkg, buildPath, { optional: true });
}

async function stageEngine(platform: string, arch: string): Promise<void> {
  if (platform !== process.platform || arch !== process.arch)
    throw new Error(
      `The engine is staged for the host only; package ${platform}/${arch} on a matching machine.`,
    );
  if (engineStatus(ENGINE_DIR).complete && existsSync(CHROMIUM_CREDITS)) return;
  await rm(ENGINE_DIR, { recursive: true, force: true });
  const log: InstallProgress = ({ part, progress, message }) =>
    process.stdout.write(
      `\r[engine] ${part} ${progress == null ? "" : percentText(progress)} ${message}   `,
    );
  await installFfmpeg(ENGINE_DIR, log);
  await installHeadlessShell(ENGINE_DIR, log);
  process.stdout.write("\n[engine] Saving the recording browser's licences\n");
  const shell = engineStatus(ENGINE_DIR).headlessShell;
  if (!shell) throw new Error("The recording browser didn't install.");
  const credits = await writeChromiumCredits(shell, CHROMIUM_CREDITS);
  process.stdout.write(
    `[engine] Chromium ${credits.version}: ${credits.bytes} bytes of licences\n`,
  );
}

async function stageEngineLicences(): Promise<void> {
  await rm(ENGINE_LICENCES_DIR, { recursive: true, force: true });
  await mkdir(ENGINE_LICENCES_DIR, { recursive: true });
  await copyFile(CHROMIUM_CREDITS, path.join(ENGINE_LICENCES_DIR, CHROMIUM_CREDITS_FILE));
}

const EXPORT_SOURCES: Array<[from: string[], to: string]> = [
  [["src", "video"], "video"],
  [["src", "shared"], "shared"],
  [["src", "renderer", "lib", "utils.ts"], "utils.ts"],
  [["public"], "public"],
];

async function stageExportSource(): Promise<void> {
  await rm(EXPORT_SRC_DIR, { recursive: true, force: true });
  await mkdir(EXPORT_SRC_DIR, { recursive: true });
  for (const [from, to] of EXPORT_SOURCES)
    await cp(path.join(ROOT, ...from), path.join(EXPORT_SRC_DIR, to), { recursive: true });
}

const LINUX_CATEGORIES: Array<"AudioVideo" | "Video"> = ["AudioVideo", "Video"];
const APPIMAGE_LAUNCHER = "demoflow-launcher";
const ICONS_DIR = path.join(ROOT, "brand", "icons");
const APPIMAGE_ICON_SIZES = [512, 256, 128, 64, 48, 32, 16];
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
  makers: [
    new MakerSquirrel(),
    new MakerZIP({}, ["darwin"]),
    new MakerRpm({ options: { categories: LINUX_CATEGORIES } }),
    new MakerDeb({
      options: { categories: LINUX_CATEGORIES, section: "video", icon: icon(512) },
    }),
    new MakerAppImage({
      options: {
        categories: LINUX_CATEGORIES,
        bin: APPIMAGE_LAUNCHER,
        icon: {
          scalable: path.join(ROOT, "brand", "logo.svg"),
          ...Object.fromEntries(APPIMAGE_ICON_SIZES.map((size) => [`${size}x${size}`, icon(size)])),
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
      await stagePackage("playwright-core", buildPath);
    },
  },
  plugins: [
    new VitePlugin({
      build: [
        { entry: "src/main/main.ts", config: "vite.main.config.ts", target: "main" },
        { entry: "src/preload/preload.ts", config: "vite.main.config.ts", target: "preload" },
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
