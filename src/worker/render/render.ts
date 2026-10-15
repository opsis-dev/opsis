import { existsSync } from "node:fs";
import path from "node:path";
import {
  makeCancelSignal,
  openBrowser,
  renderMedia,
  renderStill,
  selectComposition,
  type Codec,
  type HeadlessBrowser,
} from "@remotion/renderer";
import type { TimelineSpec } from "../../shared/schema/timeline";
import type { WorkerEnv } from "../../shared/worker-protocol";
import { enginePaths } from "../../engine/paths";
import { COMPOSITION_ID } from "../../video/composition-id";
import {
  bundleRemotionProject,
  getPrebuiltRemotionBundlePath,
  hasPrebuiltRemotionBundle,
} from "./bundle";
import { getCompositorPackage } from "./compositor-package";

export type GlRenderer = "angle" | "swangle" | "egl" | "vulkan" | "swiftshader" | "angle-egl";

let devBundle: Promise<string> | null = null;

export function remotionLicenseKey(env: WorkerEnv): string {
  return env.settings.render?.licenseKey.trim() || "free-license";
}

export function binariesDirectory(env: WorkerEnv): string | null {
  if (!env.isPackaged) return null;
  return path.join(
    path.dirname(env.appRoot),
    "app.asar.unpacked",
    "node_modules",
    getCompositorPackage({ arch: process.arch, platform: process.platform }),
  );
}

export function browserExecutable(env: WorkerEnv): string {
  const shell = enginePaths(env.engineDir).headlessShell;
  if (!existsSync(shell))
    throw new Error("The video engine isn't set up yet. Set it up in Settings → Engine.");
  return shell;
}

export async function getServeUrl(
  env: WorkerEnv,
  onProgress?: (p: number) => void,
): Promise<string> {
  if (env.isPackaged) {
    if (!hasPrebuiltRemotionBundle(env.appRoot))
      throw new Error("This copy of Demoflow is incomplete. Reinstall the app.");
    return getPrebuiltRemotionBundlePath(env.appRoot);
  }
  devBundle ??= bundleRemotionProject({
    projectRoot: env.appRoot,
    onProgress: (p) => onProgress?.(p / 100),
  });
  return devBundle;
}

export const CODEC_EXT: Record<string, string> = {
  h264: "mp4",
  h265: "mp4",
  vp8: "webm",
  vp9: "webm",
  prores: "mov",
  gif: "gif",
};

export async function renderTimeline(opts: {
  env: WorkerEnv;
  serveUrl: string;
  spec: TimelineSpec;
  output: string;
  codec: Codec;
  crf?: number;
  scale?: number;
  gl: GlRenderer;
  signal: AbortSignal;
  onProgress: (p: number) => void;
}): Promise<void> {
  const { cancel, cancelSignal } = makeCancelSignal();
  const onAbort = () => cancel();
  opts.signal.addEventListener("abort", onAbort, { once: true });
  try {
    const common = {
      serveUrl: opts.serveUrl,
      inputProps: opts.spec,
      binariesDirectory: binariesDirectory(opts.env),
      browserExecutable: browserExecutable(opts.env),
      chromiumOptions: { gl: opts.gl },
    };
    const composition = await selectComposition({ ...common, id: COMPOSITION_ID });
    await renderMedia({
      ...common,
      composition,
      codec: opts.codec,
      outputLocation: opts.output,
      crf: opts.codec === "prores" || opts.codec === "gif" ? undefined : opts.crf,
      scale: opts.scale,
      cancelSignal,
      licenseKey: remotionLicenseKey(opts.env),
      onProgress: ({ progress }) => opts.onProgress(progress),
    });
  } finally {
    opts.signal.removeEventListener("abort", onAbort);
  }
}

export async function renderFrame(opts: {
  env: WorkerEnv;
  serveUrl: string;
  spec: TimelineSpec;
  frame: number;
  output: string;
  format: "png" | "jpeg";
  gl: GlRenderer;
  signal: AbortSignal;
  browser?: HeadlessBrowser;
  scale?: number;
  jpegQuality?: number;
}): Promise<void> {
  const { cancel, cancelSignal } = makeCancelSignal();
  const onAbort = () => cancel();
  opts.signal.addEventListener("abort", onAbort, { once: true });
  try {
    const common = {
      serveUrl: opts.serveUrl,
      inputProps: opts.spec,
      binariesDirectory: binariesDirectory(opts.env),
      browserExecutable: browserExecutable(opts.env),
      chromiumOptions: { gl: opts.gl },
    };
    const shared = opts.browser ? { ...common, puppeteerInstance: opts.browser } : common;
    const composition = await selectComposition({ ...shared, id: COMPOSITION_ID });
    await renderStill({
      ...shared,
      composition,
      frame: Math.min(composition.durationInFrames - 1, Math.max(0, opts.frame)),
      output: opts.output,
      imageFormat: opts.format,
      scale: opts.scale,
      jpegQuality: opts.jpegQuality,
      cancelSignal,
      licenseKey: remotionLicenseKey(opts.env),
    });
  } finally {
    opts.signal.removeEventListener("abort", onAbort);
  }
}

export function openRenderBrowser(env: WorkerEnv, gl: GlRenderer): Promise<HeadlessBrowser> {
  return openBrowser("chrome", {
    browserExecutable: browserExecutable(env),
    chromiumOptions: { gl },
  });
}
