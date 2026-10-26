import { rm } from "node:fs/promises";
import path from "node:path";
import { VIDEO_SOURCE } from "#sidecar/source-layout";

const PREBUILT_REMOTION_BUNDLE_DIR = "remotion-bundle";
const REMOTION_ENTRY = path.join(...VIDEO_SOURCE, "index.ts");

export const getPrebuiltRemotionBundlePath = (appRoot: string) =>
  path.join(appRoot, PREBUILT_REMOTION_BUNDLE_DIR);

export async function bundleRemotionProject({
  projectRoot,
  outDir,
  onProgress,
  sourceMaps = true,
}: {
  projectRoot: string;
  outDir?: string;
  onProgress?: (percent: number) => void;
  sourceMaps?: boolean;
}): Promise<string> {
  const { bundle } = await import("@remotion/bundler");
  const { makeBundlerOverride } = await import("#sidecar/bundler-override");
  if (outDir) await rm(outDir, { recursive: true, force: true });
  return bundle({
    entryPoint: path.join(projectRoot, REMOTION_ENTRY),
    outDir,
    publicDir: path.join(projectRoot, "public"),
    rspack: true,
    rspackOverride: async (config) => {
      const overridden = await makeBundlerOverride(path.join(projectRoot, ...VIDEO_SOURCE))(config);
      if (sourceMaps) return overridden;
      const rules = overridden.module?.rules ?? [];
      return {
        ...overridden,
        devtool: false,
        module: {
          ...overridden.module,
          rules: [...rules, { test: /\.m?js$/, extractSourceMap: true }],
        },
      };
    },
    onProgress,
  });
}
