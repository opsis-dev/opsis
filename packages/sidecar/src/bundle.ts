import { rm } from "node:fs/promises";
import path from "node:path";
import { VIDEO_SOURCE } from "./source-layout";

const PREBUILT_REMOTION_BUNDLE_DIR = "remotion-bundle";
const REMOTION_ENTRY = path.join(...VIDEO_SOURCE, "index.ts");

export const getPrebuiltRemotionBundlePath = (appRoot: string) =>
  path.join(appRoot, PREBUILT_REMOTION_BUNDLE_DIR);

export async function bundleRemotionProject({
  projectRoot,
  outDir,
  onProgress,
}: {
  projectRoot: string;
  outDir?: string;
  onProgress?: (percent: number) => void;
}): Promise<string> {
  const { bundle } = await import("@remotion/bundler");
  const { makeWebpackOverride } = await import("./webpack-override");
  if (outDir) await rm(outDir, { recursive: true, force: true });
  return bundle({
    entryPoint: path.join(projectRoot, REMOTION_ENTRY),
    outDir,
    publicDir: path.join(projectRoot, "public"),
    webpackOverride: makeWebpackOverride(path.join(projectRoot, ...VIDEO_SOURCE)),
    onProgress,
  });
}
