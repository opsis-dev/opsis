import { existsSync } from "node:fs";
import { writeFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "playwright-core";
import type { z } from "zod";
import { loginDependencies, sessionChain } from "../../shared/graph";
import { sessionNode } from "../../shared/nodes/browser";
import { parseParams } from "../../shared/nodes/define";
import { nodeName } from "../../shared/nodes/registry";
import { loginOnceNode, recordNode, stillNode } from "../../shared/nodes/capture";
import type { RecordingValue } from "../../shared/schema/values";
import type { WorkflowNode } from "../../shared/schema/workflow";
import { secretName } from "../../shared/secrets";
import { addRecordingFrames } from "../../traces/inject";
import { applyRetention, ingestTrace } from "../../traces/store";
import { runTake, SignInExpiredError, type TakeOptions } from "../capture/driver";
import { FrameEncoder } from "../capture/encoder";
import { elementOf } from "../capture/flow-types";
import { resolveTarget } from "../capture/selectors";
import { defineExecutor, TracedError, type ExecutorContext } from "../engine/context";
import { rel } from "./util";
import { WINDOW_FIT } from "../../video/camera/lib/style";
import { captureScale, type Box } from "../../video/core/still-layout";
import { errorMessage } from "../../shared/errors";
import { ENGINE_MISSING_MESSAGE } from "../../shared/engine-message";
import { formatDuration } from "../../shared/format";
import { findHeadlessShell } from "../../engine/paths";

const CAPTURE_MAX_EXPORT_SCALE = 2;

const captureCanvas = (settings: { width: number; height: number }) => ({
  width: settings.width * CAPTURE_MAX_EXPORT_SCALE,
  height: settings.height * CAPTURE_MAX_EXPORT_SCALE,
});

type SignIn = { name: string; url: string; login: z.infer<typeof loginOnceNode.params> };

type SessionAccess = { secret: string | null; signIn: SignIn | undefined };

function sessionAccess(ctx: ExecutorContext, head: WorkflowNode | undefined): SessionAccess {
  if (head?.type !== sessionNode.type) return { secret: null, signIn: undefined };
  const session = parseParams(sessionNode, head, ctx.vars);
  if (!session.storageState.trim()) return { secret: null, signIn: undefined };
  const secret = secretName(session.storageState);
  const loginId = loginDependencies(ctx.workflow, ctx.node.id, ctx.vars)[0];
  const loginNode = ctx.workflow.nodes.find((n) => n.id === loginId);
  if (!loginNode) return { secret, signIn: undefined };
  const login = parseParams(loginOnceNode, loginNode, ctx.vars);
  return { secret, signIn: { name: secret, url: login.url, login } };
}

async function storageState(ctx: ExecutorContext, secret: string | null): Promise<string | null> {
  if (!secret) return null;
  const value = await ctx.secrets.read(secret);
  if (!value)
    throw new Error(`The sign-in "${secret}" isn't saved yet. Run its Sign in once node first.`);
  return value;
}

async function retryAfterSignIn<T>(ctx: ExecutorContext, attempt: () => Promise<T>): Promise<T> {
  try {
    return await attempt();
  } catch (error) {
    if (!(error instanceof TracedError && error.signInExpired) || ctx.signal.aborted) throw error;
    const { signIn } = sessionAccess(ctx, sessionChain(ctx.workflow, ctx.node.id)[0]);
    if (!signIn) throw error;
    ctx.log(
      "info",
      `The saved sign-in “${signIn.name}” has expired. Sign in again in the window that opened.`,
    );
    const state = await ctx.login({
      url: signIn.login.url,
      doneWhen: signIn.login.doneWhen,
      selector: signIn.login.selector,
      timeoutSec: signIn.login.timeoutSec,
    });
    await ctx.secrets.write(signIn.name, state);
    ctx.log("info", `Saved the new sign-in as “${signIn.name}”. Recording again.`);
    return attempt();
  }
}

function requireShell(ctx: ExecutorContext): string {
  const shell = findHeadlessShell(ctx.engine);
  if (!shell) throw new Error(ENGINE_MISSING_MESSAGE);
  return shell;
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
  const kept = await applyRetention(ctx.projectDir, ctx.settings.traces);
  return kept.has(manifest.id) ? manifest.id : undefined;
}

async function take<T>(
  ctx: ExecutorContext,
  options: Pick<TakeOptions<T>, "mode" | "encoder" | "record" | "onPage">,
) {
  const chain = sessionChain(ctx.workflow, ctx.node.id);
  const { secret, signIn } = sessionAccess(ctx, chain[0]);
  const tracePath =
    ctx.settings.traces.retention === "off" ? undefined : path.join(ctx.dir, "trace.zip");
  const traceGroups: string[] = [];
  const withFrames = async () => {
    const encoder = options.encoder;
    if (!tracePath || !encoder || encoder.originMs === null) return;
    await addRecordingFrames({
      zipFile: tracePath,
      video: encoder.output,
      originWallMs: encoder.originMs,
      workDir: ctx.dir,
      ffmpeg: ctx.ffmpeg,
    }).catch((e: unknown) =>
      ctx.log("warn", `Couldn't add screenshots to the trace: ${errorMessage(e)}`),
    );
  };
  try {
    const result = await runTake({
      ...options,
      workflow: ctx.workflow,
      chain,
      vars: ctx.vars,
      projectDir: ctx.projectDir,
      executablePath: requireShell(ctx),
      storageState: await storageState(ctx, secret),
      signIn,
      live: ctx.live(nodeName(ctx.node)),
      tracePath,
      traceGroups,
      signal: ctx.signal,
      log: ctx.log,
      step: ctx.step,
    });
    await withFrames();
    return { result, traceId: await saveTrace(ctx, tracePath, true, traceGroups) };
  } catch (error) {
    await withFrames();
    const traceId = await saveTrace(ctx, tracePath, false, traceGroups).catch(() => undefined);
    const expired = error instanceof SignInExpiredError;
    throw new TracedError(
      expired && !signIn
        ? `${error.message} Run its Sign in once node again.`
        : errorMessage(error),
      traceId,
      expired,
    );
  }
}

export const PREVIEW_WIDTH = 960;

async function makePreviewCopy(
  ctx: ExecutorContext,
  mp4: string,
  fps: number,
): Promise<string | null> {
  const name = "recording.preview.mp4";
  try {
    await ctx.ffmpeg.run(
      [
        "-i",
        mp4,
        "-an",
        "-vf",
        `scale=${PREVIEW_WIDTH}:-2`,
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-crf",
        "26",
        "-g",
        String(Math.max(1, Math.round(fps / 2))),
        "-pix_fmt",
        "yuv420p",
        "-movflags",
        "+faststart",
        path.join(ctx.dir, name),
      ],
      { signal: ctx.signal },
    );
    return name;
  } catch (error) {
    if (ctx.signal.aborted) throw error;
    ctx.log("warn", `Couldn't make the preview copy: ${errorMessage(error)}`);
    return null;
  }
}

export const recordExecutor = defineExecutor(recordNode, async (ctx, _inputs, p) => {
  const mp4 = path.join(ctx.dir, "recording.mp4");
  ctx.progress(0.02, "Starting the browser");
  const { result, traceId, frames } = await retryAfterSignIn(ctx, async () => {
    const encoder = new FrameEncoder(ctx.ffmpeg, mp4, p.fps, ctx.signal);
    const taken = await take(ctx, { mode: "record", encoder, record: p });
    return { ...taken, frames: encoder.frames };
  });
  const probe = await ctx.ffmpeg.probe(mp4);
  const log = {
    ...result.log,
    durationMs: probe.durationMs
      ? Math.min(result.log.durationMs, probe.durationMs)
      : result.log.durationMs,
  };
  await writeFile(path.join(ctx.dir, "actions.json"), JSON.stringify(log, null, 2));
  ctx.progress(0.95, "Making a preview copy");
  const proxy = await makePreviewCopy(ctx, mp4, p.fps);
  const recording: RecordingValue = {
    type: "recording",
    src: ctx.ref("recording.mp4"),
    ...(proxy ? { proxySrc: ctx.ref(proxy) } : {}),
    log,
    durationMs: log.durationMs,
    viewport: log.viewport,
    nodeId: ctx.node.id,
  };
  ctx.log(
    "info",
    `Recorded ${formatDuration(log.durationMs)}, ${frames} frames, ${log.actions.length} actions`,
  );
  return {
    outputs: { recording },
    artifacts: [
      { kind: "video", path: rel(ctx, "recording.mp4"), label: "Recording" },
      { kind: "json", path: rel(ctx, "actions.json"), label: "Action log" },
    ],
    traceId,
  };
});

type Shot = { width: number; height: number; scale: number; body?: Box };

async function captureShot(
  ctx: ExecutorContext,
  page: Page,
  p: z.infer<typeof stillNode.params>,
  file: string,
): Promise<Shot> {
  const view = await page.evaluate(() => ({
    x: window.scrollX,
    y: window.scrollY,
    width: window.innerWidth,
    height: window.innerHeight,
    pageWidth: document.documentElement.scrollWidth,
    pageHeight: document.documentElement.scrollHeight,
    dpr: window.devicePixelRatio,
  }));
  let region: Box = { x: view.x, y: view.y, width: view.width, height: view.height };
  let body: Box | undefined;
  if (p.region === "element") {
    const { target } = await resolveTarget(page, p.target, { log: ctx.log, signal: ctx.signal });
    const loc = elementOf(target);
    await loc.scrollIntoViewIfNeeded().catch(() => undefined);
    const box = await loc.boundingBox();
    if (!box) throw new Error(`Couldn't find "${p.target}" on the page for the screenshot.`);
    const scroll = await page.evaluate(() => ({ x: window.scrollX, y: window.scrollY }));
    const x = Math.max(0, box.x + scroll.x - p.padding);
    const y = Math.max(0, box.y + scroll.y - p.padding);
    region = {
      x,
      y,
      width: box.x + scroll.x + box.width + p.padding - x,
      height: box.y + scroll.y + box.height + p.padding - y,
    };
    body = {
      x: box.x + scroll.x - x,
      y: box.y + scroll.y - y,
      width: box.width,
      height: box.height,
    };
  } else if (p.region === "full-page") {
    region = { x: 0, y: 0, width: view.pageWidth, height: view.pageHeight };
  }
  const scale = captureScale(p.look, region, captureCanvas(ctx.workflow.settings), {
    body,
    fit: WINDOW_FIT,
    least: view.dpr,
  });
  const cdp = await page.context().newCDPSession(page);
  const { data } = await cdp.send("Page.captureScreenshot", {
    format: "png",
    clip: { ...region, scale },
    captureBeyondViewport:
      p.region === "full-page" || region.width > view.width || region.height > view.height,
  });
  await cdp.detach().catch(() => undefined);
  await writeFile(file, Buffer.from(data, "base64"));
  return { width: region.width, height: region.height, scale, ...(body ? { body } : {}) };
}

export const stillExecutor = defineExecutor(stillNode, async (ctx, _inputs, p) => {
  const file = path.join(ctx.dir, "still.png");
  const { result, traceId } = await retryAfterSignIn(ctx, () =>
    take(ctx, { mode: "still", onPage: (page) => captureShot(ctx, page, p, file) }),
  );
  const shot = result.fromPage;
  if (!shot) throw new Error("The screenshot wasn't taken.");
  ctx.log(
    "info",
    `Screenshot ${Math.round(shot.width * shot.scale)} × ${Math.round(shot.height * shot.scale)} px`,
  );
  return {
    outputs: { image: { type: "image", src: ctx.ref("still.png"), ...shot } },
    artifacts: [{ kind: "image", path: rel(ctx, "still.png"), label: "Screenshot" }],
    traceId,
  };
});

export const loginOnceExecutor = defineExecutor(loginOnceNode, async (ctx, _inputs, p) => {
  ctx.log("info", "Sign in in the window that opened.");
  const state = await ctx.login({
    url: p.url,
    doneWhen: p.doneWhen,
    selector: p.selector,
    timeoutSec: p.timeoutSec,
  });
  const name = secretName(p.secretName);
  await ctx.secrets.write(name, state);
  ctx.log("info", `Saved the sign-in as "${name}".`);
  return { outputs: {} };
});
