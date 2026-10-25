import { throwIfCancelled } from "@opsis/core/errors";
import type { LiveEvent } from "@opsis/core/live";
import type { recordNode } from "@opsis/core/nodes/capture";
import type { NodeParams } from "@opsis/core/nodes/define";
import { nodeName } from "@opsis/core/nodes/registry";
import type { Action } from "@opsis/core/schema/actions";
import type { BrowserSessionValue, SessionStep } from "@opsis/core/schema/values";
import type { CursorSample, KeyEvent } from "@opsis/video/capture";
import type { Locator, Page } from "playwright-core";
import { plainError, traceGroupTitle } from "./driver-helpers";
import type { FrameEncoder } from "./encoder";
import type { ActionOpts, TakeClick, TakeSink } from "./flow-types";
import { CursorDriver } from "./recorder";
import { resolveTarget, SelectorError } from "./selectors";
import type { OpenSession } from "./session";
import { clickKindOf, runStep } from "./take-steps";

export type LiveSink = {
  frame: (jpeg: Buffer, width: number, height: number, t: number) => void;
  event: (event: LiveEvent) => void;
};

type RecordOptions = Pick<
  NodeParams<typeof recordNode>,
  "fps" | "quality" | "establishMs" | "endTailMs" | "cursor" | "showActions"
>;

export type Recording = { encoder: FrameEncoder; record: RecordOptions };

export type TakeOptions = {
  session: BrowserSessionValue;
  projectDir: string;
  executablePath: string;
  live?: LiveSink | null;
  tracePath?: string;
  traceGroup?: (nodeId: string) => void;
  secrets: { read: (name: string) => Promise<string | null> };
  signal: AbortSignal;
  log: (level: "info" | "warn" | "error", message: string) => void;
  step: (nodeId: string, status: "start" | "done" | "error", message?: string) => void;
};

type MotionParams = {
  cluster: string;
  zoom: boolean;
  frame?: string;
  zoomScale?: number;
  travelMs?: number;
  hoverMs?: number;
};

const LIVE_CURSOR_GAP_MS = 30;
const FIRST_PAGE_MIN_WAIT_MS = 500;

export const atTakeStart = (action: Action): Action => ({
  ...action,
  t: 0,
  ...(action.tEnd === undefined ? {} : { tEnd: 0 }),
  ...(action.tDepart === undefined ? {} : { tDepart: 0 }),
  ...(action.tDown === undefined ? {} : { tDown: 0 }),
});

export class TakeRun implements TakeSink {
  readonly actions: Action[] = [];
  readonly keys: KeyEvent[] = [];
  readonly clicks: TakeClick[] = [];
  readonly cursorTrack: CursorSample[] = [];
  readonly hoisted = new Map<string, Locator>();
  cursorDriver: CursorDriver | null = null;
  signedIn: { url: string; name: string } | null = null;
  t0 = Date.now();
  private lastLiveCursorAt = 0;

  constructor(
    readonly opts: TakeOptions,
    readonly session: OpenSession,
    readonly page: Page,
    readonly recording: Recording | null,
  ) {}

  click(event: TakeClick) {
    this.clicks.push(event);
    const { live } = this.opts;
    live?.event({ kind: "click", x: event.x, y: event.y });
    if (event.rect)
      live?.event({
        kind: "target",
        rect: event.rect,
        label: event.label ?? "",
        selector: event.selector,
      });
  }

  cursor(sample: CursorSample) {
    this.cursorTrack.push(sample);
    const now = Date.now();
    if (now - this.lastLiveCursorAt < LIVE_CURSOR_GAP_MS) return;
    this.lastLiveCursorAt = now;
    this.opts.live?.event({ kind: "cursor", x: sample.x, y: sample.y });
  }

  key(event: KeyEvent) {
    this.keys.push(event);
  }

  elapsed = () => Date.now() - this.t0;

  rolling(): CursorDriver {
    if (this.page.url() === "about:blank")
      throw new Error("Mouse actions must come after the first Go to page action.");
    if (!this.cursorDriver) throw new Error("Mouse actions must come after Start recording.");
    return this.cursorDriver;
  }

  async pauseIfRolling(ms: number) {
    if (this.cursorDriver) await this.cursorDriver.pause(ms);
  }

  async settleAfterPage(ms: number) {
    if (this.cursorDriver) await this.cursorDriver.pause(ms);
    else await this.page.waitForTimeout(Math.max(FIRST_PAGE_MIN_WAIT_MS, ms));
  }

  async pressKey(chord: string, label: string) {
    if (this.cursorDriver) await this.cursorDriver.pressKey(chord, label);
    else await this.page.keyboard.press(chord);
  }

  resolve(raw: string, timeoutMs?: number) {
    return resolveTarget(this.page, raw, {
      log: this.opts.log,
      signal: this.opts.signal,
      hoisted: this.hoisted,
      timeoutMs,
    });
  }

  async motion(p: MotionParams, selector: string): Promise<ActionOpts> {
    return {
      selector,
      cluster: p.cluster || undefined,
      zoom: p.zoom,
      frame: p.frame ? (await this.resolve(p.frame)).target : undefined,
      zoomScale: p.zoomScale || undefined,
      travelMs: p.travelMs || undefined,
      hoverMs: p.hoverMs,
    };
  }

  pushAction(action: Omit<Action, "nodeId">, node: SessionStep) {
    this.actions.push({ ...action, nodeId: node.id });
  }

  start() {
    this.actions.splice(0, this.actions.length, ...this.actions.map(atTakeStart));
    this.t0 = Date.now();
    this.recording?.encoder.setOrigin(this.t0);
    this.cursorDriver = new CursorDriver(
      this.page,
      this.elapsed,
      this,
      this.recording?.record.cursor === "synthetic",
    );
  }

  async run(node: SessionStep) {
    const { opts } = this;
    throwIfCancelled(opts.signal);
    opts.step(node.id, "start");
    opts.live?.event({ kind: "step", nodeId: node.id, status: "running" });
    const clicksBefore = this.clicks.length;
    const keysBefore = this.keys.length;
    await this.traced(node, () => this.attempt(node));
    this.collectActions(node, clicksBefore, keysBefore);
    opts.step(node.id, "done");
    opts.live?.event({ kind: "step", nodeId: node.id, status: "done" });
  }

  private async traced(node: SessionStep, play: () => Promise<void>) {
    const { opts } = this;
    if (!opts.tracePath) return play();
    await this.session.context.tracing.group(traceGroupTitle(nodeName(node), node));
    opts.traceGroup?.(node.id);
    try {
      await play();
    } finally {
      await this.session.context.tracing.groupEnd().catch(() => undefined);
    }
  }

  private async attempt(node: SessionStep) {
    const { opts } = this;
    try {
      await runStep(this, node);
    } catch (raw) {
      const error = raw instanceof SelectorError ? raw : new Error(plainError(raw, node));
      opts.step(node.id, "error", error.message);
      opts.live?.event({ kind: "step", nodeId: node.id, status: "error", message: error.message });
      throw error;
    }
  }

  private collectActions(node: SessionStep, clicksBefore: number, keysBefore: number) {
    for (const c of this.clicks.slice(clicksBefore))
      this.pushAction(this.clickAction(c, node), node);
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
  }

  private clickAction(c: TakeClick, node: SessionStep): Omit<Action, "nodeId"> {
    const kind = clickKindOf(node.type);
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
      selectorUsed: c.selector,
      zoom: c.zoom,
      zoomScale: c.zoomScale,
      text: kind === "type" ? c.text : undefined,
      path: c.path,
      drop: c.drop,
    };
  }
}
