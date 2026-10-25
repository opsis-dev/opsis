import { existsSync } from "node:fs";
import { writeFile } from "node:fs/promises";
import path from "node:path";
import { PREVIEW_FILE_WIDTH } from "@opsis/core/compile/recording";
import { errorMessage } from "@opsis/core/errors";
import { formatDuration, plural } from "@opsis/core/format";
import { openBrowserParams, signInNode } from "@opsis/core/nodes/browser";
import { recordNode, stillNode } from "@opsis/core/nodes/capture";
import type { NodeParams } from "@opsis/core/nodes/define";
import { SESSION_PORT } from "@opsis/core/nodes/ports";
import { nodeName } from "@opsis/core/nodes/registry";
import { asValue, type BrowserSessionValue, type RecordingValue } from "@opsis/core/schema/values";
import { secretName } from "@opsis/core/secrets";
import { atLeastOneFrame } from "@opsis/core/timeline/time";
import {
  captureCanvas,
  captureScale,
  paddedRegion,
  tallestShot,
  WINDOW_FIT,
  type Box,
} from "@opsis/video/capture";
import type { Locator, Page } from "playwright-core";
import { writeJson } from "../../engine/json";
import { requireHeadlessShell } from "../../engine/paths";
import { addRecordingFrames } from "../../traces/inject";
import { applyRetention, ingestTrace } from "../../traces/store";
import { recordTake, stillTake } from "../capture/driver";
import { FrameEncoder } from "../capture/encoder";
import { elementOf } from "../capture/flow-types";
import { resolveTarget } from "../capture/selectors";
import type { TakeOptions } from "../capture/take-run";
import {
  defineExecutor,
  TRACE_NOT_SAVED,
  TracedError,
  warnFailed,
  type ExecutorContext,
  type ExecutorInputs,
} from "../engine/context";
import { h264Args } from "../media/codecs";
import { projectFileRef, rel } from "./util";

const RECORDING_FILE = "recording.mp4";
const PREVIEW_FILE = "recording.preview.mp4";
const ACTIONS_FILE = "actions.json";
const STILL_FILE = "still.png";
const TRACE_FILE = "trace.zip";

function sessionInput(inputs: ExecutorInputs): BrowserSessionValue {
  const session = asValue(inputs[SESSION_PORT], "browser-session");
  if (!session) throw new Error("Connect Open browser and its actions to this node.");
  return session;
}

async function saveTrace(
  ctx: ExecutorContext,
  file: string | undefined,
  ok: boolean,
  groups: string[],
): Promise<string | undefined> {
  if (!file || !existsSync(file)) return undefined;
  const manifest = await ingestTrace(ctx.projectDir, file, {
    nodeId: ctx.node.id,
    label: nodeName(ctx.node),
    ok,
    groups,
  });
  const kept = await applyRetention(ctx.projectDir, ctx.env.settings.traces).catch((e: unknown) => {
    warnFailed(ctx.log, "Couldn't remove old traces", e);
    return new Set([manifest.id]);
  });
  return kept.has(manifest.id) ? manifest.id : undefined;
}

type TraceFile = { path: string | undefined; groups: string[] };

async function addFrames(ctx: ExecutorContext, file: string, encoder: FrameEncoder) {
  if (encoder.originMs === null) return;
  await addRecordingFrames({
    zipFile: file,
    video: encoder.output,
    originWallMs: encoder.originMs,
    workDir: ctx.dir,
    ffmpeg: ctx.ffmpeg,
  }).then(
    (frames) => {
      if (!frames.added)
        ctx.log("info", `Screenshots weren't added to the trace: ${frames.reason}.`);
    },
    (e: unknown) => warnFailed(ctx.log, "Couldn't add screenshots to the trace", e),
  );
}

async function finalizeTrace(
  ctx: ExecutorContext,
  trace: TraceFile,
  encoder: FrameEncoder | undefined,
  ok: boolean,
): Promise<string | undefined> {
  try {
    if (trace.path && encoder) await addFrames(ctx, trace.path, encoder);
    return await saveTrace(ctx, trace.path, ok, trace.groups);
  } catch (error) {
    warnFailed(ctx.log, TRACE_NOT_SAVED, error);
    return undefined;
  }
}

async function runTakeWithTrace<R>(
  ctx: ExecutorContext,
  session: BrowserSessionValue,
  encoder: FrameEncoder | undefined,
  play: (opts: TakeOptions) => Promise<R>,
): Promise<{ result: R; traceId: string | undefined }> {
  const trace: TraceFile = {
    path: ctx.env.settings.traces.retention === "off" ? undefined : path.join(ctx.dir, TRACE_FILE),
    groups: [],
  };
  let result: R;
  try {
    result = await play({
      session,
      projectDir: ctx.projectDir,
      executablePath: requireHeadlessShell(ctx.engine),
      secrets: ctx.secrets,
      live: ctx.live(nodeName(ctx.node)),
      tracePath: trace.path,
      traceGroup: (nodeId) => trace.groups.push(nodeId),
      signal: ctx.signal,
      log: ctx.log,
      step: ctx.step,
    });
  } catch (error) {
    const traceId = await finalizeTrace(ctx, trace, encoder, false);
    throw new TracedError(errorMessage(error), traceId);
  }
  return { result, traceId: await finalizeTrace(ctx, trace, encoder, true) };
}

const PREVIEW_CRF = 26;
const PREVIEW_KEYFRAMES_PER_SECOND = 2;

async function makePreviewCopy(
  ctx: ExecutorContext,
  mp4: string,
  fps: number,
): Promise<string | null> {
  try {
    await ctx.ffmpeg.run(
      [
        "-i",
        mp4,
        "-an",
        "-vf",
        `scale=${PREVIEW_FILE_WIDTH}:-2`,
        ...h264Args({
          crf: PREVIEW_CRF,
          preset: "veryfast",
          keyframeEvery: atLeastOneFrame(fps / PREVIEW_KEYFRAMES_PER_SECOND),
          faststart: true,
        }),
        path.join(ctx.dir, PREVIEW_FILE),
      ],
      { signal: ctx.signal },
    );
    return PREVIEW_FILE;
  } catch (error) {
    if (ctx.signal.aborted) throw error;
    warnFailed(ctx.log, "Couldn't make the preview copy", error);
    return null;
  }
}

export const recordExecutor = defineExecutor(recordNode, async (ctx, inputs, p) => {
  const session = sessionInput(inputs);
  const mp4 = path.join(ctx.dir, RECORDING_FILE);
  ctx.progress(0.02, "Starting the browser");
  const encoder = new FrameEncoder(ctx.ffmpeg, mp4, p.fps, ctx.signal);
  const { result, traceId } = await runTakeWithTrace(ctx, session, encoder, (opts) =>
    recordTake(opts, { encoder, record: p }),
  );
  const { frames } = encoder;
  const probe = await ctx.ffmpeg.probe(mp4);
  const log = {
    ...result.log,
    durationMs: probe.durationMs
      ? Math.min(result.log.durationMs, probe.durationMs)
      : result.log.durationMs,
  };
  await writeJson(path.join(ctx.dir, ACTIONS_FILE), log);
  ctx.progress(0.95, "Making a preview copy");
  const proxy = await makePreviewCopy(ctx, mp4, p.fps);
  const recording: RecordingValue = {
    type: "recording",
    src: projectFileRef(ctx, RECORDING_FILE),
    ...(proxy ? { proxySrc: projectFileRef(ctx, proxy) } : {}),
    log,
    durationMs: log.durationMs,
    viewport: log.viewport,
    nodeId: ctx.node.id,
    style: [],
  };
  ctx.log(
    "info",
    `Recorded ${formatDuration(log.durationMs)}, ${plural(frames, "frame")}, ${plural(log.actions.length, "action")}`,
  );
  return {
    outputs: { recording },
    artifacts: [
      { kind: "video", path: rel(ctx, RECORDING_FILE), label: "Recording" },
      { kind: "json", path: rel(ctx, ACTIONS_FILE), label: "Action log" },
    ],
    traceId,
  };
});

type Shot = { width: number; height: number; scale: number; body?: Box };

type View = {
  x: number;
  y: number;
  width: number;
  height: number;
  pageWidth: number;
  pageHeight: number;
  dpr: number;
};

type ShotRegion = { region: Box; body?: Box };

type StillParams = NodeParams<typeof stillNode>;

const viewOf = (page: Page): Promise<View> =>
  page.evaluate(() => ({
    x: window.scrollX,
    y: window.scrollY,
    width: window.innerWidth,
    height: window.innerHeight,
    pageWidth: document.documentElement.scrollWidth,
    pageHeight: document.documentElement.scrollHeight,
    dpr: window.devicePixelRatio,
  }));

const SIZE_TRIES = 20;
const SIZE_POLL_MS = 50;

async function sizedBox(page: Page, loc: Locator) {
  let box = await loc.boundingBox();
  for (let i = 0; i < SIZE_TRIES && box && (box.width < 1 || box.height < 1); i++) {
    await page.waitForTimeout(SIZE_POLL_MS);
    box = await loc.boundingBox();
  }
  return box;
}

async function elementRegion(
  ctx: ExecutorContext,
  page: Page,
  p: StillParams,
): Promise<ShotRegion> {
  const { target } = await resolveTarget(page, p.target, { log: ctx.log, signal: ctx.signal });
  const loc = elementOf(target);
  await loc.scrollIntoViewIfNeeded().catch(() => undefined);
  const box = await sizedBox(page, loc);
  if (!box) throw new Error(`Couldn't find “${p.target}” on the page for the screenshot.`);
  if (box.width < 1 || box.height < 1)
    throw new Error(
      `“${p.target}” has no size on the page, so there's nothing to capture. Add space around it, or pick another element.`,
    );
  const scroll = await page.evaluate(() => ({ x: window.scrollX, y: window.scrollY }));
  return paddedRegion(box, scroll, p.padding);
}

async function regionFor(
  ctx: ExecutorContext,
  page: Page,
  p: StillParams,
  view: View,
): Promise<ShotRegion> {
  if (p.region === "element") return elementRegion(ctx, page, p);
  if (p.region === "full-page") {
    const tallest = tallestShot(p.look, view.pageWidth, captureCanvas(ctx.workflow.settings));
    if (view.pageHeight > tallest)
      ctx.log(
        "info",
        `The page is ${view.pageHeight} px tall; keeping the top ${tallest} px, as much as the video can show.`,
      );
    return {
      region: { x: 0, y: 0, width: view.pageWidth, height: Math.min(view.pageHeight, tallest) },
    };
  }
  return { region: { x: view.x, y: view.y, width: view.width, height: view.height } };
}

async function captureShot(
  ctx: ExecutorContext,
  page: Page,
  p: StillParams,
  file: string,
): Promise<Shot> {
  const view = await viewOf(page);
  const { region, body } = await regionFor(ctx, page, p, view);
  const scale = captureScale(p.look, region, captureCanvas(ctx.workflow.settings), {
    body,
    fit: WINDOW_FIT,
    least: view.dpr,
  });
  const cdp = await page.context().newCDPSession(page);
  const shot = await cdp
    .send("Page.captureScreenshot", {
      format: "png",
      clip: { ...region, scale },
      captureBeyondViewport:
        p.region === "full-page" || region.width > view.width || region.height > view.height,
    })
    .finally(() => cdp.detach().catch(() => undefined));
  await writeFile(file, Buffer.from(shot.data, "base64"));
  return { width: region.width, height: region.height, scale, ...(body ? { body } : {}) };
}

export const stillExecutor = defineExecutor(stillNode, async (ctx, inputs, p) => {
  const file = path.join(ctx.dir, STILL_FILE);
  const { result, traceId } = await runTakeWithTrace(ctx, sessionInput(inputs), undefined, (opts) =>
    stillTake({ ...opts, onPage: (page) => captureShot(ctx, page, p, file) }),
  );
  const shot = result.fromPage;
  ctx.log(
    "info",
    `Screenshot ${Math.round(shot.width * shot.scale)} × ${Math.round(shot.height * shot.scale)} px`,
  );
  return {
    outputs: { image: { type: "image", src: projectFileRef(ctx, STILL_FILE), ...shot } },
    artifacts: [{ kind: "image", path: rel(ctx, STILL_FILE), label: "Screenshot" }],
    traceId,
  };
});

export const signInExecutor = defineExecutor(signInNode, async (ctx, inputs, p) => {
  const name = secretName(p.secretName);
  if (!ctx.forced && (await ctx.secrets.read(name))) {
    ctx.log("info", `Using the saved sign-in “${name}”.`);
    return { outputs: {} };
  }
  const session = asValue(inputs[SESSION_PORT], "browser-session");
  const browser = session && openBrowserParams(session);
  ctx.log("info", "Sign in in the window that opened.");
  const state = await ctx.login({
    url: p.url,
    doneWhen: p.doneWhen,
    selector: p.selector,
    timeoutSec: p.timeoutSec,
    ...(browser
      ? { viewport: { width: browser.viewportWidth, height: browser.viewportHeight } }
      : {}),
  });
  await ctx.secrets.write(name, state);
  ctx.log("info", `Saved the sign-in as “${name}”.`);
  return { outputs: {} };
});
