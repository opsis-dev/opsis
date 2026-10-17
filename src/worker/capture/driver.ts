import path from "node:path";
import type { Locator, Page } from "playwright-core";
import {
  assertNode,
  chapterNode,
  clickNode,
  dragNode,
  freezeClockNode,
  hideElementsNode,
  hoistNode,
  hoverNode,
  mockRouteNode,
  navigateNode,
  overlayHtmlNode,
  pressNode,
  scrollNode,
  selectNode,
  sessionNode,
  setStateNode,
  typeNode,
  uploadNode,
  waitNode,
} from "../../shared/nodes/browser";
import { parseParams, type NodeDef } from "../../shared/nodes/define";
import { nodeLabelOr, nodeName } from "../../shared/nodes/registry";
import type { LiveEvent } from "../../shared/live";
import type { Action, ActionLog } from "../../shared/schema/actions";
import type { Workflow, WorkflowNode } from "../../shared/schema/workflow";
import { PAUSE_MARK } from "../../shared/vars";
import { errorMessage } from "../../shared/errors";
import { nonBlankLines } from "../../shared/format";
import type { CursorSample, KeyEvent } from "../../video/camera/lib/click-log";
import { parsePath } from "./drag-path";
import { landedOnSignIn, plainError, setupLength, traceGroupTitle } from "./driver-helpers";
import type { FrameEncoder } from "./encoder";
import { elementOf, type ActionOpts, type ClickEvent, type DragTo } from "./flow-types";
import {
  addStyles,
  beforeFirstLoad,
  CHAT_WIDGET_SELECTORS,
  COOKIE_BANNER_SELECTORS,
  CURSOR_HOTSPOT,
  hidingCss,
  installCursor,
} from "./page-scripts";
import { CursorDriver } from "./recorder";
import { dumpVisible, resolveTarget, SelectorError } from "./selectors";
import { openSession, type OpenSession } from "./session";

type TakeMode = "record" | "still" | "check";

export type LiveSink = {
  frame: (jpeg: Buffer, width: number, height: number, t: number) => void;
  event: (event: LiveEvent) => void;
};

type RecordOptions = {
  fps: number;
  quality: number;
  establishMs: number;
  endTailMs: number;
  cursor: "synthetic" | "baked" | "none";
  showActions: boolean;
};

export type TakeOptions<T = void> = {
  workflow: Workflow;
  chain: WorkflowNode[];
  vars: Record<string, string>;
  mode: TakeMode;
  projectDir: string;
  executablePath: string;
  encoder?: FrameEncoder;
  live?: LiveSink | null;
  tracePath?: string;
  traceGroups?: string[];
  signIn?: { url: string; name: string };
  record?: RecordOptions;
  onPage?: (page: Page) => Promise<T>;
  storageState?: string | null;
  signal: AbortSignal;
  log: (level: "info" | "warn" | "error", message: string) => void;
  step: (nodeId: string, status: "start" | "done" | "error", message?: string) => void;
};

type TakeResult<T> = {
  log: ActionLog;
  t0: number;
  physicalViewport: { width: number; height: number };
  fromPage?: T;
};

const DEFAULT_FRAME_QUALITY = 80;
const SHOW_ACTIONS_MS = 600;
const LIVE_CURSOR_GAP_MS = 30;
const FIRST_PAGE_MIN_WAIT_MS = 500;
const SCROLL_TICK_MS = 16;

const ACTION_KINDS: Partial<Record<string, Action["kind"]>> = {
  "browser.type": "type",
  "browser.click": "click",
  "browser.hover": "hover",
  "browser.drag": "drag",
};

export class SignInExpiredError extends Error {
  constructor(readonly secretName: string) {
    super(`The saved sign-in “${secretName}” has expired.`);
    this.name = "SignInExpiredError";
  }
}

type MotionParams = {
  cluster: string;
  zoom: boolean;
  frame?: string;
  zoomScale?: number;
  travelMs?: number;
  hoverMs?: number;
};

function observed<T>(onPush: (item: T) => void): T[] {
  const list: T[] = [];
  const push = list.push.bind(list);
  Object.defineProperty(list, "push", {
    enumerable: false,
    value: (...items: T[]) => {
      for (const item of items) onPush(item);
      return push(...items);
    },
  });
  return list;
}

class TakeRun {
  readonly actions: Action[] = [];
  readonly keys: KeyEvent[] = [];
  readonly clicks: ClickEvent[];
  readonly cursor: CursorSample[];
  readonly hoisted = new Map<string, Locator>();
  readonly recording: boolean;
  lastSelector: string | undefined;
  lastTyped: string | undefined;
  cursorDriver: CursorDriver | null = null;
  t0 = Date.now();
  private lastCursorAt = 0;

  constructor(
    readonly opts: TakeOptions<unknown>,
    readonly session: OpenSession,
    readonly page: Page,
    readonly record: RecordOptions | undefined,
  ) {
    this.recording = !!record;
    const live = opts.live;
    this.clicks = observed<ClickEvent>((c) => {
      live?.event({ kind: "click", x: c.x, y: c.y });
      if (c.rect)
        live?.event({
          kind: "target",
          rect: c.rect,
          label: c.label ?? "",
          selector: this.lastSelector,
        });
    });
    this.cursor = observed<CursorSample>((c) => {
      const now = Date.now();
      if (now - this.lastCursorAt < LIVE_CURSOR_GAP_MS) return;
      this.lastCursorAt = now;
      live?.event({ kind: "cursor", x: c.x, y: c.y });
    });
  }

  get vars() {
    return this.opts.vars;
  }

  params<S extends NodeDef["params"]>(def: NodeDef<S>, node: WorkflowNode) {
    return parseParams(def, node, this.vars);
  }

  elapsed = () => Date.now() - this.t0;

  rolling(): CursorDriver {
    if (!this.cursorDriver)
      throw new Error("Mouse actions must come after the first Go to page action.");
    return this.cursorDriver;
  }

  async pauseIfRolling(ms: number) {
    if (this.cursorDriver) await this.cursorDriver.pause(ms);
  }

  resolve(raw: string, timeoutMs?: number) {
    return resolveTarget(this.page, raw, {
      log: this.opts.log,
      signal: this.opts.signal,
      hoisted: this.hoisted,
      timeoutMs,
    });
  }

  async motion(p: MotionParams): Promise<ActionOpts> {
    return {
      cluster: p.cluster || undefined,
      zoom: p.zoom,
      frame: p.frame ? (await this.resolve(p.frame)).target : undefined,
      zoomScale: p.zoomScale || undefined,
      travelMs: p.travelMs || undefined,
      hoverMs: p.hoverMs,
    };
  }

  pushAction(action: Omit<Action, "nodeId">, node: WorkflowNode) {
    this.actions.push({ ...action, nodeId: node.id });
  }

  start() {
    this.t0 = Date.now();
    this.opts.encoder?.setOrigin(this.t0);
    this.cursorDriver = new CursorDriver(
      this.page,
      this.clicks,
      this.elapsed,
      this.cursor,
      this.keys,
    );
  }

  async run(node: WorkflowNode) {
    const { opts } = this;
    if (opts.signal.aborted) throw new Error("Cancelled");
    opts.step(node.id, "start");
    opts.live?.event({ kind: "step", nodeId: node.id, status: "running" });
    const clicksBefore = this.clicks.length;
    const keysBefore = this.keys.length;
    if (opts.tracePath) {
      await this.session.context.tracing.group(traceGroupTitle(nodeName(node), node, this.vars));
      opts.traceGroups?.push(node.id);
    }
    try {
      await runStep(this, node);
    } catch (raw) {
      const error =
        raw instanceof SelectorError || raw instanceof SignInExpiredError
          ? raw
          : new Error(plainError(raw, node, this.vars));
      opts.step(node.id, "error", error.message);
      opts.live?.event({ kind: "step", nodeId: node.id, status: "error", message: error.message });
      throw error;
    } finally {
      if (opts.tracePath) await this.session.context.tracing.groupEnd().catch(() => undefined);
    }
    for (const c of this.clicks.slice(clicksBefore)) this.actions.push(this.clickAction(c, node));
    for (const k of this.keys.slice(keysBefore)) {
      this.pushAction(
        {
          id: `${node.id}:k${this.actions.length}`,
          kind: "key",
          t: k.tMs,
          text: k.chord,
          label: k.label,
        },
        node,
      );
    }
    opts.step(node.id, "done");
    opts.live?.event({ kind: "step", nodeId: node.id, status: "done" });
  }

  private clickAction(c: ClickEvent, node: WorkflowNode): Action {
    const kind = ACTION_KINDS[node.type] ?? "focus";
    return {
      id: `${node.id}:${this.actions.length}`,
      kind,
      t: c.tMs,
      tEnd: c.typeEndMs,
      tDepart: c.tDepartMs,
      tDown: c.tDownMs,
      point: { x: c.x, y: c.y },
      rect: c.rect,
      cluster: c.cluster,
      label: c.label,
      selectorUsed: this.lastSelector,
      zoom: c.zoom,
      zoomScale: c.zoomScale,
      text: kind === "type" ? this.lastTyped : undefined,
      nodeId: node.id,
      ...(c.path ? { path: c.path } : {}),
      ...(c.drop ? { drop: c.drop } : {}),
    };
  }
}

type StepHandler = (take: TakeRun, node: WorkflowNode) => Promise<void>;

const STEPS: Record<string, StepHandler> = {
  "browser.navigate": async (take, node) => {
    const p = take.params(navigateNode, node);
    const t = take.elapsed();
    await take.page.goto(p.url, { waitUntil: p.waitUntil });
    take.pushAction(
      { id: `${node.id}:nav`, kind: "navigate", t, tEnd: take.elapsed(), label: p.url },
      node,
    );
    if (take.cursorDriver) await take.cursorDriver.pause(p.after);
    else await take.page.waitForTimeout(Math.max(FIRST_PAGE_MIN_WAIT_MS, p.after));
    const { signIn } = take.opts;
    if (signIn && landedOnSignIn(p.url, take.page.url(), signIn.url))
      throw new SignInExpiredError(signIn.name);
  },
  "browser.click": async (take, node) => {
    const p = take.params(clickNode, node);
    const t = await take.resolve(p.target);
    await take.rolling().moveAndClick(t.target, nodeLabelOr(node, p.target), await take.motion(p));
    take.lastSelector = t.by;
    await take.rolling().pause(p.after);
  },
  "browser.type": async (take, node) => {
    const p = take.params(typeNode, node);
    const t = await take.resolve(p.target);
    const chunks = p.text.split(PAUSE_MARK);
    await take.rolling().typeInto(t.target, chunks, nodeLabelOr(node, p.target), {
      ...(await take.motion(p)),
      chunkPauseMs: p.chunkPauseMs,
    });
    take.lastSelector = t.by;
    take.lastTyped = chunks.join("");
    if (p.pressEnter) await take.rolling().pressKey("Enter", "Enter");
    await take.rolling().pause(p.after);
  },
  "browser.hover": async (take, node) => {
    const p = take.params(hoverNode, node);
    const t = await take.resolve(p.target);
    if (p.zoom) {
      await take.rolling().focus(t.target, nodeLabelOr(node, p.target), await take.motion(p));
    } else {
      await take.rolling().moveTo(t.target);
      await take.rolling().pause(p.hoverMs);
    }
    take.lastSelector = t.by;
    await take.rolling().pause(p.after);
  },
  "browser.drag": async (take, node) => {
    const p = take.params(dragNode, node);
    const from = await take.resolve(p.target);
    if (p.mode === "element" && !p.to.trim()) throw new Error("Pick where to drop it.");
    const to: DragTo =
      p.mode === "element"
        ? { target: (await take.resolve(p.to)).target }
        : p.mode === "by"
          ? { by: { x: p.dx, y: p.dy } }
          : { path: parsePath(p.path) };
    await take.rolling().drag(from.target, to, nodeLabelOr(node, p.target), {
      ...(await take.motion(p)),
      holdMs: p.holdMs,
      dragMs: p.dragMs,
      method: p.method,
    });
    take.lastSelector = from.by;
    await take.rolling().pause(p.after);
  },
  "browser.scroll": async (take, node) => {
    const p = take.params(scrollNode, node);
    const { page } = take;
    const t = take.elapsed();
    if (p.mode === "element") {
      const loc = elementOf((await take.resolve(p.target)).target);
      if (p.smooth) {
        await loc.evaluate((el) => el.scrollIntoView({ behavior: "smooth", block: "center" }));
        await page.waitForTimeout(p.durationMs);
      } else await loc.scrollIntoViewIfNeeded();
    } else if (p.smooth) {
      const ticks = Math.max(1, Math.round(p.durationMs / SCROLL_TICK_MS));
      for (let i = 0; i < ticks; i++) {
        await page.mouse.wheel(0, p.deltaY / ticks);
        await page.waitForTimeout(SCROLL_TICK_MS);
      }
    } else {
      await page.evaluate((dy) => window.scrollBy(0, dy), p.deltaY);
    }
    take.pushAction(
      {
        id: `${node.id}:scroll`,
        kind: "scroll",
        t,
        tEnd: take.elapsed(),
        label: node.label?.trim() || undefined,
      },
      node,
    );
    await take.pauseIfRolling(p.after);
  },
  "browser.select": async (take, node) => {
    const p = take.params(selectNode, node);
    const t = await take.resolve(p.target);
    await take.rolling().focus(t.target, nodeLabelOr(node, p.target), await take.motion(p));
    const loc = elementOf(t.target);
    await loc.selectOption(p.value).catch(async () => loc.selectOption({ label: p.value }));
    take.lastSelector = t.by;
    await take.rolling().pause(p.after);
  },
  "browser.upload": async (take, node) => {
    const p = take.params(uploadNode, node);
    const t = await take.resolve(p.target);
    await elementOf(t.target).setInputFiles(path.resolve(take.opts.projectDir, p.file));
    await take.pauseIfRolling(p.after);
  },
  "browser.press": async (take, node) => {
    const p = take.params(pressNode, node);
    if (take.cursorDriver) await take.cursorDriver.pressKey(p.keys, nodeLabelOr(node, p.keys));
    else await take.page.keyboard.press(p.keys);
    await take.pauseIfRolling(p.after);
  },
  "browser.wait": async (take, node) => {
    const p = take.params(waitNode, node);
    const { page } = take;
    const t = take.elapsed();
    if (p.mode === "element") await take.resolve(p.target, p.timeoutMs);
    else if (p.mode === "network-idle")
      await page.waitForLoadState("networkidle", { timeout: p.timeoutMs });
    else if (p.mode === "url") await page.waitForURL(p.url, { timeout: p.timeoutMs });
    else await page.waitForTimeout(p.ms);
    take.pushAction({ id: `${node.id}:wait`, kind: "wait", t, tEnd: take.elapsed() }, node);
  },
  "browser.assert": async (take, node) => {
    const p = take.params(assertNode, node);
    const scope = p.target ? elementOf((await take.resolve(p.target)).target) : take.page;
    try {
      await scope.getByText(p.text).first().waitFor({ state: "visible", timeout: p.timeoutMs });
    } catch {
      throw new Error(`"${p.text}" isn't on the page.\n${await dumpVisible(take.page)}`);
    }
  },
  "browser.hoist": async (take, node) => {
    const p = take.params(hoistNode, node);
    const t = await take.resolve(p.target);
    take.hoisted.set(p.target, elementOf(t.target));
  },
  "browser.setState": async (take, node) => {
    const p = take.params(setStateNode, node);
    if (take.page.url() === "about:blank") {
      await take.session.context.addInitScript({ content: beforeFirstLoad(node.id, p.script) });
      return;
    }
    await take.page.evaluate(`(async () => { ${p.script}\n })()`);
    if (p.reload) await take.page.reload();
  },
  "browser.mockRoute": async (take, node) => {
    const p = take.params(mockRouteNode, node);
    await take.page.route(p.url, (route) =>
      route.fulfill({ status: p.status, contentType: p.contentType, body: p.body }),
    );
  },
  "browser.freezeClock": async (take, node) => {
    const p = take.params(freezeClockNode, node);
    const time = new Date(p.time);
    if (p.mode === "fixed") await take.page.clock.setFixedTime(time);
    else {
      await take.page.clock.install({ time });
      await take.page.clock.pauseAt(time);
    }
  },
  "browser.hideElements": async (take, node) => {
    const p = take.params(hideElementsNode, node);
    const selectors = [
      ...(p.cookieBanners ? COOKIE_BANNER_SELECTORS : []),
      ...(p.chatWidgets ? CHAT_WIDGET_SELECTORS : []),
      ...nonBlankLines(p.selectors),
    ];
    if (!selectors.length) return;
    const css = hidingCss(selectors);
    await take.session.context.addInitScript(addStyles, css);
    await take.page.evaluate(addStyles, css).catch(() => undefined);
  },
  "browser.chapter": async (take, node) => {
    const p = take.params(chapterNode, node);
    take.pushAction(
      {
        id: `${node.id}:chapter`,
        kind: "chapter",
        t: take.elapsed(),
        label: p.title,
        text: p.description || undefined,
      },
      node,
    );
    if (p.showInVideo && take.recording) {
      await take.page.screencast.showChapter(p.title, {
        description: p.description || undefined,
        duration: p.durationMs,
      });
      await take.page.waitForTimeout(p.durationMs);
    }
  },
  "browser.overlayHtml": async (take, node) => {
    const p = take.params(overlayHtmlNode, node);
    if (!take.recording) return;
    const overlay = await take.page.screencast.showOverlay(p.html);
    if (p.durationMs > 0)
      setTimeout(() => void overlay.dispose().catch(() => undefined), p.durationMs);
  },
  "browser.session": async () => {
    throw new Error("Only one Open browser node is allowed per take.");
  },
};

function runStep(take: TakeRun, node: WorkflowNode): Promise<void> {
  const handler = STEPS[node.type];
  if (!handler)
    throw new Error(`"${nodeName({ type: node.type })}" can't be part of a browser take.`);
  return handler(take, node);
}

async function startScreencast(take: TakeRun, live: LiveSink | null) {
  const { page, session, opts, record } = take;
  await page.screencast.start({
    size: session.physicalViewport,
    quality: record?.quality ?? DEFAULT_FRAME_QUALITY,
    onFrame: (f) => {
      live?.frame(f.data, f.viewportWidth, f.viewportHeight, f.timestamp);
      opts.encoder?.offer(f.data, f.timestamp);
    },
  });
  if (record?.showActions)
    await page.screencast.showActions({ cursor: "pointer", duration: SHOW_ACTIONS_MS });
}

async function stopTake(take: TakeRun, failed: boolean, screencasting: boolean): Promise<number> {
  const { opts, page, session } = take;
  const end = Date.now();
  let durationMs = end - take.t0;
  if (screencasting)
    await page.screencast
      .stop()
      .catch((e: unknown) =>
        opts.log("warn", `The recording didn't stop cleanly: ${errorMessage(e)}`),
      );
  if (opts.encoder) {
    if (opts.signal.aborted) await opts.encoder.abort();
    else {
      const finished = opts.encoder.finish(end);
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

export async function runTake<T>(opts: TakeOptions<T>): Promise<TakeResult<T>> {
  const { chain, vars, mode, signal } = opts;
  const head = chain[0];
  if (head?.type !== sessionNode.type)
    throw new Error("A take must start with an Open browser node.");
  const sessionParams = parseParams(sessionNode, head, vars);
  const steps = chain.slice(1).filter((n) => !n.disabled);
  const live = opts.live ?? null;
  const record = mode === "record" && opts.encoder ? opts.record : undefined;

  opts.step(head.id, "start");
  const session = await openSession(sessionParams, {
    storageState: opts.storageState,
    executablePath: opts.executablePath,
  });
  const onAbort = () => void session.close();
  signal.addEventListener("abort", onAbort, { once: true });

  if (opts.tracePath)
    await session.context.tracing.start({ screenshots: !record, snapshots: true, sources: false });
  const page = await session.context.newPage();
  if (record?.cursor === "baked")
    await session.context.addInitScript(installCursor, CURSOR_HOTSPOT);
  opts.step(head.id, "done");

  live?.event({
    kind: "start",
    title: nodeName(head),
    viewport: session.physicalViewport,
    steps: steps.map((n) => ({ nodeId: n.id, label: nodeName(n) })),
  });
  const take = new TakeRun(opts, session, page, record);
  const screencasting = !!record || !!live;
  let failed = false;
  let fromPage: T | undefined;
  let durationMs = 0;
  try {
    if (screencasting) await startScreencast(take, live);
    const setupEnd = setupLength(steps, vars);
    for (const node of steps.slice(0, setupEnd)) await take.run(node);
    take.start();
    if (record) await page.waitForTimeout(record.establishMs);
    for (const node of steps.slice(setupEnd)) await take.run(node);
    if (record) await page.waitForTimeout(record.endTailMs);
    else if (mode === "still" && opts.onPage) fromPage = await opts.onPage(page);
  } catch (error) {
    failed = true;
    if (error instanceof SelectorError) opts.log("error", error.message);
    live?.event({ kind: "end", ok: false, message: errorMessage(error) });
    throw error;
  } finally {
    durationMs = await stopTake(take, failed, screencasting);
    signal.removeEventListener("abort", onAbort);
    await session.close();
  }
  live?.event({ kind: "end", ok: true });

  return {
    t0: take.t0,
    physicalViewport: session.physicalViewport,
    fromPage,
    log: {
      viewport: session.physicalViewport,
      dpr: sessionParams.dpr,
      t0: take.t0,
      durationMs,
      actions: take.actions,
      cursorTrack: opts.record?.cursor === "synthetic" ? [...take.cursor] : undefined,
    },
  };
}
