import type { Page } from "playwright-core";
import { sessionNode } from "../../shared/nodes/browser";
import { parseNodeParams } from "../../shared/nodes/define";
import { nodeName } from "../../shared/nodes/registry";
import type { ActionLog } from "../../shared/schema/actions";
import type { WorkflowNode } from "../../shared/schema/workflow";
import { errorMessage, onAbort } from "../../shared/errors";
import { setupLength } from "./driver-helpers";
import { CURSOR_LOOK, installCursor } from "./page-scripts";
import { SelectorError } from "./selectors";
import { openSession, type OpenSession } from "./session";
import { TakeRun, type Recording, type TakeOptions } from "./take-run";

type RecordTake = TakeOptions & Recording;

type StillTake<T> = TakeOptions & { onPage: (page: Page) => Promise<T> };

type TakeResult<T> = {
  log: ActionLog;
  t0: number;
  physicalViewport: { width: number; height: number };
  fromPage: T;
};

type OpenTake = {
  take: TakeRun;
  steps: WorkflowNode[];
  dpr: number;
  stopOnAbort: () => void;
};

const DEFAULT_FRAME_QUALITY = 80;
const SHOW_ACTIONS_MS = 600;

async function startScreencast(take: TakeRun) {
  const { page, session, opts, recording } = take;
  const record = recording?.record;
  await page.screencast.start({
    size: session.physicalViewport,
    quality: record?.quality ?? DEFAULT_FRAME_QUALITY,
    onFrame: (f) => {
      opts.live?.frame(f.data, f.viewportWidth, f.viewportHeight, f.timestamp);
      recording?.encoder.offer(f.data, f.timestamp);
    },
  });
  if (record?.showActions)
    await page.screencast.showActions({ cursor: "pointer", duration: SHOW_ACTIONS_MS });
}

async function stopTake(take: TakeRun, failed: boolean, screencasting: boolean): Promise<number> {
  const { opts, page, session, recording } = take;
  const end = Date.now();
  let durationMs = end - take.t0;
  if (screencasting)
    await page.screencast
      .stop()
      .catch((e: unknown) =>
        opts.log("warn", `The recording didn't stop cleanly: ${errorMessage(e)}`),
      );
  if (recording) {
    if (opts.signal.aborted) await recording.encoder.abort();
    else {
      const finished = recording.encoder.finish(end);
      if (failed) await finished.catch(() => undefined);
      else durationMs = await finished;
    }
  }
  if (opts.tracePath)
    await session.context.tracing
      .stop({ path: opts.tracePath })
      .catch((e: unknown) => opts.log("warn", `Couldn't save the trace: ${errorMessage(e)}`));
  return durationMs;
}

async function preparePage(session: OpenSession, opts: TakeOptions, recording: Recording | null) {
  if (opts.tracePath)
    await session.context.tracing.start({
      screenshots: !recording,
      snapshots: true,
      sources: false,
    });
  const page = await session.context.newPage();
  if (recording?.record.cursor === "baked")
    await session.context.addInitScript(installCursor, CURSOR_LOOK);
  return page;
}

async function openTake(opts: TakeOptions, recording: Recording | null): Promise<OpenTake> {
  const { chain, vars, signal } = opts;
  const head = chain[0];
  if (head?.type !== sessionNode.type)
    throw new Error("A take must start with an Open browser node.");
  const sessionParams = parseNodeParams(sessionNode, head, vars);
  const steps = chain.slice(1).filter((n) => !n.disabled);

  opts.step(head.id, "start");
  const session = await openSession(sessionParams, {
    storageState: opts.storageState,
    executablePath: opts.executablePath,
  });
  const stopOnAbort = onAbort(signal, () => void session.close());
  let page: Page;
  try {
    page = await preparePage(session, opts, recording);
  } catch (error) {
    stopOnAbort();
    await session.close();
    throw error;
  }
  opts.step(head.id, "done");

  opts.live?.event({
    kind: "start",
    title: nodeName(head),
    viewport: session.physicalViewport,
    steps: steps.map((n) => ({ nodeId: n.id, label: nodeName(n) })),
  });
  return {
    take: new TakeRun(opts, session, page, recording),
    steps,
    dpr: sessionParams.dpr,
    stopOnAbort,
  };
}

async function playChain(take: TakeRun, steps: WorkflowNode[]) {
  const { record } = take.recording ?? {};
  const setupEnd = setupLength(steps, take.vars);
  for (const node of steps.slice(0, setupEnd)) await take.run(node);
  take.start();
  if (record) await take.page.waitForTimeout(record.establishMs);
  for (const node of steps.slice(setupEnd)) await take.run(node);
}

function takeLog(take: TakeRun, dpr: number, durationMs: number): ActionLog {
  const { physicalViewport } = take.session;
  return {
    viewport: physicalViewport,
    dpr,
    t0: take.t0,
    durationMs,
    actions: take.actions,
    cursorTrack: take.recording?.record.cursor === "synthetic" ? [...take.cursorTrack] : undefined,
  };
}

async function playTake<T>(
  opts: TakeOptions,
  recording: Recording | null,
  finale: (take: TakeRun) => Promise<T>,
): Promise<TakeResult<T>> {
  const { take, steps, dpr, stopOnAbort } = await openTake(opts, recording);
  const screencasting = !!recording || !!opts.live;
  let failed = false;
  let durationMs = 0;
  let fromPage: T;
  try {
    if (screencasting) await startScreencast(take);
    await playChain(take, steps);
    fromPage = await finale(take);
  } catch (error) {
    failed = true;
    if (error instanceof SelectorError) opts.log("error", error.message);
    opts.live?.event({ kind: "end", ok: false, message: errorMessage(error) });
    throw error;
  } finally {
    durationMs = await stopTake(take, failed, screencasting);
    stopOnAbort();
    await take.session.close();
  }
  opts.live?.event({ kind: "end", ok: true });
  return {
    t0: take.t0,
    physicalViewport: take.session.physicalViewport,
    fromPage,
    log: takeLog(take, dpr, durationMs),
  };
}

export const recordTake = (opts: RecordTake): Promise<TakeResult<void>> =>
  playTake(opts, { encoder: opts.encoder, record: opts.record }, (take) =>
    take.page.waitForTimeout(opts.record.endTailMs),
  );

export const stillTake = <T>(opts: StillTake<T>): Promise<TakeResult<T>> =>
  playTake(opts, null, (take) => opts.onPage(take.page));
