import { existsSync } from "node:fs";
import { rm } from "node:fs/promises";
import path from "node:path";
import type { BundleOptions } from "@remotion/bundler";

export const PREBUILT_REMOTION_BUNDLE_DIR = "remotion-bundle";
export const REMOTION_ENTRY = path.join("src", "video", "index.ts");

type BundleProgress = Parameters<NonNullable<BundleOptions["onProgress"]>>[0];

export const getPrebuiltRemotionBundlePath = (appRoot: string) =>
  path.join(appRoot, PREBUILT_REMOTION_BUNDLE_DIR);

export const hasPrebuiltRemotionBundle = (appRoot: string) =>
  existsSync(getPrebuiltRemotionBundlePath(appRoot));

export async function bundleRemotionProject({
  projectRoot,
  outDir,
  onProgress,
}: {
  projectRoot: string;
  outDir?: string;
  onProgress?: (progress: BundleProgress) => void;
}): Promise<string> {
  const { bundle } = await import("@remotion/bundler");
  const { makeWebpackOverride } = await import("./webpack-override");
  if (outDir) await rm(outDir, { recursive: true, force: true });
  return bundle({
    entryPoint: path.join(projectRoot, REMOTION_ENTRY),
    outDir,
    publicDir: path.join(projectRoot, "public"),
    webpackOverride: makeWebpackOverride(projectRoot),
    onProgress,
  });
}
