import { existsSync } from "node:fs";
import path from "node:path";
import {
  makeCancelSignal,
  openBrowser,
  renderMedia,
  renderStill,
  selectComposition,
  type HeadlessBrowser,
} from "@remotion/renderer";
import type { TimelineSpec } from "../../shared/schema/timeline";
import type { WorkerEnv } from "../../shared/worker-protocol";
import { enginePaths, findHeadlessShell } from "../../engine/paths";
import { COMPOSITION_ID } from "../../video/composition-id";
import { bundleRemotionProject, getPrebuiltRemotionBundlePath } from "./bundle";
import { CODECS, type RenderCodec } from "./codecs";
import { getCompositorPackage } from "./compositor-package";
import { ENGINE_MISSING_MESSAGE } from "../../shared/engine-message";
import { clamp } from "../../shared/math";

export type GlRenderer = "angle" | "swangle" | "egl" | "vulkan" | "swiftshader" | "angle-egl";

type CancelSignal = ReturnType<typeof makeCancelSignal>["cancelSignal"];

let devBundle: Promise<string> | null = null;

export function remotionLicenseKey(env: WorkerEnv): string {
  return env.settings.render.licenseKey.trim() || "free-license";
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
  const shell = findHeadlessShell(enginePaths(env.engineDir));
  if (!shell) throw new Error(ENGINE_MISSING_MESSAGE);
  return shell;
}

export async function getServeUrl(
  env: WorkerEnv,
  onProgress?: (p: number) => void,
): Promise<string> {
  if (env.isPackaged) {
    const prebuilt = getPrebuiltRemotionBundlePath(env.appRoot);
    if (!existsSync(prebuilt))
      throw new Error("This copy of Demoflow is incomplete. Reinstall the app.");
    return prebuilt;
  }
  devBundle ??= bundleRemotionProject({
    projectRoot: env.appRoot,
    onProgress: (p) => onProgress?.(p / 100),
  }).catch((error: unknown) => {
    devBundle = null;
    throw error;
  });
  return devBundle;
}

async function withCancel<T>(
  signal: AbortSignal,
  run: (cancelSignal: CancelSignal) => Promise<T>,
): Promise<T> {
  const { cancel, cancelSignal } = makeCancelSignal();
  const onAbort = () => cancel();
  signal.addEventListener("abort", onAbort, { once: true });
  try {
    return await run(cancelSignal);
  } finally {
    signal.removeEventListener("abort", onAbort);
  }
}

const renderOptions = (env: WorkerEnv, serveUrl: string, spec: TimelineSpec, gl: GlRenderer) => ({
  serveUrl,
  inputProps: spec,
  binariesDirectory: binariesDirectory(env),
  browserExecutable: browserExecutable(env),
  chromiumOptions: { gl },
});

export function renderTimeline(opts: {
  env: WorkerEnv;
  serveUrl: string;
  spec: TimelineSpec;
  output: string;
  codec: RenderCodec;
  crf: number;
  scale: number;
  gl: GlRenderer;
  muted: boolean;
  signal: AbortSignal;
  onProgress: (p: number) => void;
}): Promise<void> {
  return withCancel(opts.signal, async (cancelSignal) => {
    const common = renderOptions(opts.env, opts.serveUrl, opts.spec, opts.gl);
    const composition = await selectComposition({ ...common, id: COMPOSITION_ID });
    await renderMedia({
      ...common,
      composition,
      codec: opts.codec,
      outputLocation: opts.output,
      crf: CODECS[opts.codec].crf ? opts.crf : undefined,
      scale: opts.scale,
      muted: opts.muted,
      concurrency: 1,
      cancelSignal,
      licenseKey: remotionLicenseKey(opts.env),
      onProgress: ({ progress }) => opts.onProgress(progress),
    });
  });
}

export function renderFrame(opts: {
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
  return withCancel(opts.signal, async (cancelSignal) => {
    const common = renderOptions(opts.env, opts.serveUrl, opts.spec, opts.gl);
    const shared = opts.browser ? { ...common, puppeteerInstance: opts.browser } : common;
    const composition = await selectComposition({ ...shared, id: COMPOSITION_ID });
    await renderStill({
      ...shared,
      composition,
      frame: clamp(opts.frame, 0, composition.durationInFrames - 1),
      output: opts.output,
      imageFormat: opts.format,
      scale: opts.scale,
      jpegQuality: opts.jpegQuality,
      cancelSignal,
      licenseKey: remotionLicenseKey(opts.env),
    });
  });
}

export function openRenderBrowser(env: WorkerEnv, gl: GlRenderer): Promise<HeadlessBrowser> {
  return openBrowser("chrome", {
    browserExecutable: browserExecutable(env),
    chromiumOptions: { gl },
  });
}
