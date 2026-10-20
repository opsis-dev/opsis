import { existsSync } from "node:fs";
import { writeFile } from "node:fs/promises";
import path from "node:path";
import type { Locator, Page } from "playwright-core";
import { writeJson } from "../../engine/json";
import { requireHeadlessShell } from "../../engine/paths";
import { errorMessage } from "../../shared/errors";
import { formatDuration, plural } from "../../shared/format";
import { loginDependencies, sessionChain } from "../../shared/graph";
import { findById } from "../../shared/lists";
import { sessionNode } from "../../shared/nodes/browser";
import { loginOnceNode, recordNode, stillNode } from "../../shared/nodes/capture";
import { parseNodeParams, type NodeParams } from "../../shared/nodes/define";
import { nodeName } from "../../shared/nodes/registry";
import type { RecordingValue } from "../../shared/schema/values";
import type { WorkflowNode } from "../../shared/schema/workflow";
import { secretName } from "../../shared/secrets";
import { atLeastOneFrame } from "../../shared/timeline/time";
import type { LoginRequest } from "../../shared/worker-protocol";
import { addRecordingFrames } from "../../traces/inject";
import { applyRetention, ingestTrace } from "../../traces/store";
import { WINDOW_FIT } from "../../video/camera/lib/style";
import {
  captureCanvas,
  captureScale,
  paddedRegion,
  tallestShot,
  type Box,
} from "../../video/core/still-layout";
import { recordTake, stillTake } from "../capture/driver";
import { FrameEncoder } from "../capture/encoder";
import { elementOf } from "../capture/flow-types";
import { resolveTarget } from "../capture/selectors";
import type { TakeOptions } from "../capture/take-run";
import { SignInExpiredError } from "../capture/take-steps";
import {
  defineExecutor,
  TRACE_NOT_SAVED,
  TracedError,
  warnFailed,
  type ExecutorContext,
} from "../engine/context";
import { h264Args } from "../media/codecs";
import { projectFileRef, rel } from "./util";

const RECORDING_FILE = "recording.mp4";
const PREVIEW_FILE = "recording.preview.mp4";
const ACTIONS_FILE = "actions.json";
const STILL_FILE = "still.png";
const TRACE_FILE = "trace.zip";

type LoginParams = NodeParams<typeof loginOnceNode>;

type SignIn = { url: string; request: LoginRequest };

const loginRequest = (p: LoginParams): LoginRequest => ({
  url: p.url,
  doneWhen: p.doneWhen,
  selector: p.selector,
  timeoutSec: p.timeoutSec,
});

type SessionAccess = { secret: string | null; signIn: SignIn | undefined };

const NO_ACCESS: SessionAccess = { secret: null, signIn: undefined };

function sessionAccess(ctx: ExecutorContext, head: WorkflowNode | undefined): SessionAccess {
  if (head?.type !== sessionNode.type) return NO_ACCESS;
  const session = parseNodeParams(sessionNode, head, ctx.vars);
  if (!session.storageState.trim()) return NO_ACCESS;
  const secret = secretName(session.storageState);
  const loginId = loginDependencies(ctx.workflow, ctx.node.id, ctx.vars)[0];
  const loginNode = findById(ctx.workflow.nodes, loginId);
  if (!loginNode) return { secret, signIn: undefined };
  const login = parseNodeParams(loginOnceNode, loginNode, ctx.vars);
  return { secret, signIn: { url: login.url, request: loginRequest(login) } };
}

async function storageState(ctx: ExecutorContext, secret: string | null): Promise<string | null> {
  if (!secret) return null;
  const value = await ctx.secrets.read(secret);
  if (!value)
    throw new Error(`The sign-in “${secret}” isn't saved yet. Run its Sign in once node first.`);
  return value;
}

type Session = { chain: WorkflowNode[]; access: SessionAccess };

async function retryAfterSignIn<T>(
  ctx: ExecutorContext,
  attempt: (session: Session) => Promise<T>,
): Promise<T> {
  const chain = sessionChain(ctx.workflow, ctx.node.id);
  const session = { chain, access: sessionAccess(ctx, chain[0]) };
  try {
    return await attempt(session);
  } catch (error) {
    if (!(error instanceof TracedError && error.signInExpired) || ctx.signal.aborted) throw error;
    const { secret, signIn } = session.access;
    if (!signIn || !secret) throw error;
    ctx.log(
      "info",
      `The saved sign-in “${secret}” has expired. Sign in again in the window that opened.`,
    );
    const state = await ctx.login(signIn.request);
    await ctx.secrets.write(secret, state);
    ctx.log("info", `Saved the new sign-in as “${secret}”. Recording again.`);
    return attempt(session);
  }
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
  { chain, access }: Session,
  encoder: FrameEncoder | undefined,
  play: (opts: TakeOptions) => Promise<R>,
): Promise<{ result: R; traceId: string | undefined }> {
  const { secret, signIn } = access;
  const trace: TraceFile = {
    path: ctx.env.settings.traces.retention === "off" ? undefined : path.join(ctx.dir, TRACE_FILE),
    groups: [],
  };
  let result: R;
  try {
    result = await play({
      workflow: ctx.workflow,
      chain,
      vars: ctx.vars,
      projectDir: ctx.projectDir,
      executablePath: requireHeadlessShell(ctx.engine),
      storageState: await storageState(ctx, secret),
      signIn: signIn && secret ? { url: signIn.url, name: secret } : undefined,
      live: ctx.live(nodeName(ctx.node)),
      tracePath: trace.path,
      traceGroup: (nodeId) => trace.groups.push(nodeId),
      signal: ctx.signal,
      log: ctx.log,
      step: ctx.step,
    });
  } catch (error) {
    const traceId = await finalizeTrace(ctx, trace, encoder, false);
    const expired = error instanceof SignInExpiredError;
    throw new TracedError(
      expired && !signIn
        ? `${error.message} Run its Sign in once node again.`
        : errorMessage(error),
      traceId,
      expired,
    );
  }
  return { result, traceId: await finalizeTrace(ctx, trace, encoder, true) };
}

const PREVIEW_WIDTH = 960;
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
        `scale=${PREVIEW_WIDTH}:-2`,
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

export const recordExecutor = defineExecutor(recordNode, async (ctx, _inputs, p) => {
  const mp4 = path.join(ctx.dir, RECORDING_FILE);
  ctx.progress(0.02, "Starting the browser");
  const { result, traceId, frames } = await retryAfterSignIn(ctx, async (session) => {
    const encoder = new FrameEncoder(ctx.ffmpeg, mp4, p.fps, ctx.signal);
    const taken = await runTakeWithTrace(ctx, session, encoder, (opts) =>
      recordTake(opts, { encoder, record: p }),
    );
    return { ...taken, frames: encoder.frames };
  });
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

export const stillExecutor = defineExecutor(stillNode, async (ctx, _inputs, p) => {
  const file = path.join(ctx.dir, STILL_FILE);
  const { result, traceId } = await retryAfterSignIn(ctx, (session) =>
    runTakeWithTrace(ctx, session, undefined, (opts) =>
      stillTake({ ...opts, onPage: (page) => captureShot(ctx, page, p, file) }),
    ),
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

export const loginOnceExecutor = defineExecutor(loginOnceNode, async (ctx, _inputs, p) => {
  ctx.log("info", "Sign in in the window that opened.");
  const state = await ctx.login(loginRequest(p));
  const name = secretName(p.secretName);
  await ctx.secrets.write(name, state);
  ctx.log("info", `Saved the sign-in as “${name}”.`);
  return { outputs: {} };
});
