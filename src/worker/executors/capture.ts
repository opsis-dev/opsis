import { existsSync } from "node:fs";
import { writeFile } from "node:fs/promises";
import path from "node:path";
import { loginDependencies, sessionChain } from "../../shared/graph";
import { sessionNode } from "../../shared/nodes/browser";
import { getNodeDef } from "../../shared/nodes/registry";
import { loginOnceNode, recordNode, stillNode } from "../../shared/nodes/capture";
import type { RecordingValue } from "../../shared/schema/values";
import type { WorkflowNode } from "../../shared/schema/workflow";
import { interpolate } from "../../shared/vars";
import { addRecordingFrames } from "../../traces/inject";
import { applyRetention, ingestTrace } from "../../traces/store";
import { runTake, SignInExpiredError, type TakeOptions } from "../capture/driver";
import { FrameEncoder } from "../capture/encoder";
import { defineExecutor, TracedError, type ExecutorContext } from "../engine/context";
import { rel } from "./util";

async function storageStateFor(ctx: ExecutorContext, head: WorkflowNode): Promise<string | null> {
  const p = sessionNode.params.parse(interpolate(head.params ?? {}, ctx.vars));
  if (!p.storageState) return null;
  const value = await ctx.secrets.read(p.storageState);
  if (!value)
    throw new Error(
      `The sign-in "${p.storageState}" isn't saved yet. Run its Sign in once node first.`,
    );
  return value;
}

function signInFor(ctx: ExecutorContext, head: WorkflowNode | undefined) {
  if (!head) return undefined;
  const session = sessionNode.params.parse(interpolate(head.params ?? {}, ctx.vars));
  if (!session.storageState) return undefined;
  const loginId = loginDependencies(ctx.workflow, ctx.node.id, ctx.vars)[0];
  const loginNode = ctx.workflow.nodes.find((n) => n.id === loginId);
  if (!loginNode) return undefined;
  const login = loginOnceNode.params.parse(interpolate(loginNode.params ?? {}, ctx.vars));
  return { name: session.storageState, url: login.url, login };
}

async function retryAfterSignIn<T>(ctx: ExecutorContext, attempt: () => Promise<T>): Promise<T> {
  try {
    return await attempt();
  } catch (error) {
    if (!(error instanceof TracedError && error.signInExpired) || ctx.signal.aborted) throw error;
    const signIn = signInFor(ctx, sessionChain(ctx.workflow, ctx.node.id)[0]);
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
  if (!existsSync(ctx.engine.headlessShell))
    throw new Error("The video engine isn't set up yet. Set it up in Settings → Engine.");
  return ctx.engine.headlessShell;
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
    label: ctx.node.label || getNodeDef(ctx.node.type)?.title || ctx.node.type,
    ok,
    groups,
  });
  await applyRetention(ctx.projectDir, ctx.settings.traces);
  return existsSync(
    path.join(ctx.projectDir, ".demoflow", "traces", "runs", `${manifest.id}.json.zst`),
  )
    ? manifest.id
    : undefined;
}

async function take(
  ctx: ExecutorContext,
  options: Omit<
    TakeOptions,
    | "executablePath"
    | "workflow"
    | "chain"
    | "vars"
    | "projectDir"
    | "signal"
    | "log"
    | "step"
    | "live"
    | "storageState"
    | "tracePath"
    | "traceGroups"
  >,
) {
  const chain = sessionChain(ctx.workflow, ctx.node.id);
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
      ctx.log("warn", `Couldn't add screenshots to the trace: ${String(e)}`),
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
      storageState: await storageStateFor(ctx, chain[0]),
      signIn: signInFor(ctx, chain[0]),
      live: ctx.live(ctx.node.label || getNodeDef(ctx.node.type)?.title || ctx.node.type),
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
      expired && !signInFor(ctx, chain[0])
        ? `${error.message} Run its Sign in once node again.`
        : error instanceof Error
          ? error.message
          : String(error),
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
    ctx.log("warn", `Couldn't make the preview copy: ${String(error)}`);
    return null;
  }
}

export const recordExecutor = defineExecutor(recordNode, async (ctx, _inputs, p) => {
  const mp4 = path.join(ctx.dir, "recording.mp4");
  ctx.progress(0.02, "Starting the browser");
  let encoder: FrameEncoder | undefined;
  const { result, traceId } = await retryAfterSignIn(ctx, () => {
    encoder = new FrameEncoder(ctx.ffmpeg, mp4, p.fps, ctx.signal);
    return take(ctx, { mode: "record", encoder, record: p });
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
    `Recorded ${(log.durationMs / 1000).toFixed(1)}s, ${encoder?.frames ?? 0} frames, ${log.actions.length} actions`,
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

export const stillExecutor = defineExecutor(stillNode, async (ctx, _inputs, p) => {
  const file = path.join(ctx.dir, "still.png");
  let size = { width: 0, height: 0 };
  const { traceId } = await retryAfterSignIn(ctx, () =>
    take(ctx, {
      mode: "still",
      onPage: async (page) => {
        if (p.region === "element") {
          const loc = p.target.startsWith("css=")
            ? page.locator(p.target.slice(4)).first()
            : page.getByText(p.target).first();
          const box = await loc.boundingBox();
          if (!box) throw new Error(`Couldn't find "${p.target}" on the page for the screenshot.`);
          const pad = p.padding;
          await page.screenshot({
            path: file,
            clip: {
              x: Math.max(0, box.x - pad),
              y: Math.max(0, box.y - pad),
              width: box.width + pad * 2,
              height: box.height + pad * 2,
            },
          });
          size = { width: box.width + pad * 2, height: box.height + pad * 2 };
        } else {
          await page.screenshot({ path: file, fullPage: p.region === "full-page" });
          size = page.viewportSize() ?? size;
        }
      },
    }),
  );
  return {
    outputs: {
      image: { type: "image", src: ctx.ref("still.png"), width: size.width, height: size.height },
    },
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
  await ctx.secrets.write(p.secretName, state);
  ctx.log("info", `Saved the sign-in as "${p.secretName}".`);
  return { outputs: {} };
});
