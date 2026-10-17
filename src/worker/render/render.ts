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
import { CODECS, type RenderCodec } from "../media/codecs";
import { getCompositorPackage } from "./compositor-package";
import { ENGINE_MISSING_MESSAGE } from "../../shared/engine";
import { clamp } from "../../shared/math";
import { keepLatest } from "../../shared/memo";
import { onAbort } from "../../shared/errors";

export type GlRenderer = "angle" | "swangle" | "egl" | "vulkan" | "swiftshader" | "angle-egl";

type CancelSignal = ReturnType<typeof makeCancelSignal>["cancelSignal"];

const devBundle = keepLatest<string>();

function remotionLicenseKey(env: WorkerEnv): string {
  return env.settings.render.licenseKey.trim() || "free-license";
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

function browserExecutable(env: WorkerEnv): string {
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
  return devBundle(env.appRoot, () =>
    bundleRemotionProject({
      projectRoot: env.appRoot,
      onProgress: (p) => onProgress?.(p / 100),
    }),
  );
}

async function withCancel<T>(
  signal: AbortSignal,
  run: (cancelSignal: CancelSignal) => Promise<T>,
): Promise<T> {
  const { cancel, cancelSignal } = makeCancelSignal();
  const stopListening = onAbort(signal, cancel);
  try {
    return await run(cancelSignal);
  } finally {
    stopListening();
  }
}

type RenderJob = {
  env: WorkerEnv;
  serveUrl: string;
  spec: TimelineSpec;
  gl: GlRenderer;
  signal: AbortSignal;
  browser?: HeadlessBrowser;
};

async function compositionFor(job: RenderJob) {
  const common = {
    serveUrl: job.serveUrl,
    inputProps: job.spec,
    binariesDirectory: binariesDirectory(job.env),
    browserExecutable: browserExecutable(job.env),
    chromiumOptions: { gl: job.gl },
    ...(job.browser ? { puppeteerInstance: job.browser } : {}),
  };
  const composition = await selectComposition({ ...common, id: COMPOSITION_ID });
  return { ...common, composition, licenseKey: remotionLicenseKey(job.env) };
}

type Prepared = Awaited<ReturnType<typeof compositionFor>> & { cancelSignal: CancelSignal };

const withComposition = (job: RenderJob, render: (prepared: Prepared) => Promise<unknown>) =>
  withCancel(job.signal, async (cancelSignal) => {
    await render({ ...(await compositionFor(job)), cancelSignal });
  });

export function renderTimeline(
  opts: RenderJob & {
    output: string;
    codec: RenderCodec;
    crf: number;
    scale: number;
    muted: boolean;
    onProgress: (p: number) => void;
  },
): Promise<void> {
  return withComposition(opts, (prepared) =>
    renderMedia({
      ...prepared,
      codec: opts.codec,
      outputLocation: opts.output,
      crf: CODECS[opts.codec].crf ? opts.crf : undefined,
      scale: opts.scale,
      muted: opts.muted,
      concurrency: 1,
      onProgress: ({ progress }) => opts.onProgress(progress),
    }),
  );
}

export function renderFrame(
  opts: RenderJob & {
    frame: number;
    output: string;
    format: "png" | "jpeg";
    scale?: number;
    jpegQuality?: number;
  },
): Promise<void> {
  return withComposition(opts, (prepared) =>
    renderStill({
      ...prepared,
      frame: clamp(opts.frame, 0, prepared.composition.durationInFrames - 1),
      output: opts.output,
      imageFormat: opts.format,
      scale: opts.scale,
      jpegQuality: opts.jpegQuality,
    }),
  );
}

export function openRenderBrowser(env: WorkerEnv, gl: GlRenderer): Promise<HeadlessBrowser> {
  return openBrowser("chrome", {
    browserExecutable: browserExecutable(env),
    chromiumOptions: { gl },
  });
}
