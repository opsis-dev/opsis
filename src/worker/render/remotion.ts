import { existsSync } from "node:fs";
import path from "node:path";
import { nonBlankString } from "@opsis/core/guards";
import { keepLatest } from "@opsis/core/memo";
import type { TimelineSpec } from "@opsis/core/schema/timeline";
import type { WorkerEnv } from "@opsis/core/worker-protocol";
import { bundleRemotionProject, getPrebuiltRemotionBundlePath } from "@opsis/sidecar/bundle";
import { getCompositorPackage } from "@opsis/sidecar/compositor-package";
import type { Gl, RenderTarget } from "@opsis/sidecar/generated/protocol";
import { COMPOSITION_ID } from "@opsis/video/composition-id";
import { enginePaths, requireHeadlessShell } from "../../engine/paths";

const devBundle = keepLatest<string>();

function remotionLicenseKey(env: WorkerEnv): string {
  return nonBlankString(env.settings.render.licenseKey) ?? "free-license";
}

function binariesDirectory(env: WorkerEnv): string | null {
  if (!env.isPackaged) return null;
  return path.join(
    path.dirname(env.appRoot),
    "app.asar.unpacked",
    "node_modules",
    getCompositorPackage({ arch: process.arch, platform: process.platform }),
  );
}

export async function getServeUrl(
  env: WorkerEnv,
  onProgress?: (p: number) => void,
): Promise<string> {
  if (env.isPackaged) {
    const prebuilt = getPrebuiltRemotionBundlePath(env.appRoot);
    if (!existsSync(prebuilt))
      throw new Error("This copy of Opsis is incomplete. Reinstall the app.");
    return prebuilt;
  }
  return devBundle(env.appRoot, () =>
    bundleRemotionProject({
      projectRoot: env.appRoot,
      onProgress: (p) => onProgress?.(p / 100),
    }),
  );
}

export const renderTarget = (
  env: WorkerEnv,
  serveUrl: string,
  spec: TimelineSpec,
  gl: Gl,
): RenderTarget => ({
  serveUrl,
  browserExecutable: requireHeadlessShell(enginePaths(env.engineDir)),
  binariesDirectory: binariesDirectory(env),
  licenseKey: remotionLicenseKey(env),
  composition: COMPOSITION_ID,
  spec,
  gl,
});
