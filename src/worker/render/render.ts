import { existsSync } from "node:fs";
import path from "node:path";
import {
  makeCancelSignal,
  openBrowser,
  renderMedia,
  renderStill,
  selectComposition,
  type ChromiumOptions,
  type HeadlessBrowser,
} from "@remotion/renderer";
import { freeTcpPort } from "../../engine/free-tcp-port";
import { enginePaths, requireHeadlessShell } from "../../engine/paths";
import { onAbort } from "../../shared/errors";
import { nonBlankString } from "../../shared/guards";
import { clamp } from "../../shared/math";
import { keepLatest } from "../../shared/memo";
import type { TimelineSpec } from "../../shared/schema/timeline";
import type { WorkerEnv } from "../../shared/worker-protocol";
import { COMPOSITION_ID } from "../../video/composition-id";
import { CODECS, type RenderCodec } from "../media/codecs";
import { bundleRemotionProject, getPrebuiltRemotionBundlePath } from "./bundle";
import { getCompositorPackage } from "./compositor-package";

export type GlRenderer = NonNullable<ChromiumOptions["gl"]>;

type CancelSignal = ReturnType<typeof makeCancelSignal>["cancelSignal"];

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

const browserExecutable = (env: WorkerEnv): string =>
  requireHeadlessShell(enginePaths(env.engineDir));

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
  const composition = await selectComposition({
    ...common,
    id: COMPOSITION_ID,
    port: await freeTcpPort(),
  });
  return {
    ...common,
    port: await freeTcpPort(),
    composition,
    licenseKey: remotionLicenseKey(job.env),
  };
}

type Prepared = Awaited<ReturnType<typeof compositionFor>> & { cancelSignal: CancelSignal };

async function withComposition(
  job: RenderJob,
  render: (prepared: Prepared) => Promise<unknown>,
): Promise<void> {
  const { cancel, cancelSignal } = makeCancelSignal();
  const stopListening = onAbort(job.signal, cancel);
  try {
    await render({ ...(await compositionFor(job)), cancelSignal });
  } finally {
    stopListening();
  }
}

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
