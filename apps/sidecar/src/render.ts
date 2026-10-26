import { makeCancelSignal, renderMedia, renderStill, selectComposition } from "@remotion/renderer";
import { freeTcpPort } from "#sidecar/free-tcp-port";
import type { MediaJob, RenderTarget, StillJob } from "#sidecar/generated/protocol";

type CancelSignal = ReturnType<typeof makeCancelSignal>["cancelSignal"];

export type RenderControl = { signal: AbortSignal };

async function compositionFor(target: RenderTarget) {
  const common = {
    serveUrl: target.serveUrl,
    inputProps: target.spec,
    binariesDirectory: target.binariesDirectory,
    browserExecutable: target.browserExecutable,
    chromiumOptions: { gl: target.gl },
  };
  const composition = await selectComposition({
    ...common,
    id: target.composition,
    port: await freeTcpPort(),
  });
  return {
    ...common,
    port: await freeTcpPort(),
    composition,
    licenseKey: target.licenseKey,
  };
}

type Prepared = Awaited<ReturnType<typeof compositionFor>> & { cancelSignal: CancelSignal };

async function withComposition(
  target: RenderTarget,
  control: RenderControl,
  render: (prepared: Prepared) => Promise<unknown>,
): Promise<void> {
  const { cancel, cancelSignal } = makeCancelSignal();
  control.signal.addEventListener("abort", cancel, { once: true });
  try {
    await render({ ...(await compositionFor(target)), cancelSignal });
  } finally {
    control.signal.removeEventListener("abort", cancel);
  }
}

export function renderTimeline(
  job: MediaJob,
  control: RenderControl & { onProgress: (p: number) => void },
): Promise<void> {
  return withComposition(job, control, (prepared) =>
    renderMedia({
      ...prepared,
      codec: job.codec,
      outputLocation: job.output,
      crf: job.crf,
      scale: job.scale,
      muted: job.muted,
      concurrency: 1,
      onProgress: ({ progress }) => control.onProgress(progress),
    }),
  );
}

export function renderFrame(job: StillJob, control: RenderControl): Promise<void> {
  return withComposition(job, control, (prepared) =>
    renderStill({
      ...prepared,
      frame: Math.min(Math.max(job.frame, 0), prepared.composition.durationInFrames - 1),
      output: job.output,
      imageFormat: job.format,
      scale: job.scale,
      jpegQuality: job.jpegQuality,
    }),
  );
}
