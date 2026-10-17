import type { Locator, Page } from "playwright-core";
import { parseParams, type NodeDef } from "../../shared/nodes/define";
import { nodeName } from "../../shared/nodes/registry";
import type { LiveEvent } from "../../shared/live";
import type { Action } from "../../shared/schema/actions";
import type { Workflow, WorkflowNode } from "../../shared/schema/workflow";
import { throwIfCancelled } from "../../shared/errors";
import type { CursorSample, KeyEvent } from "../../video/camera/lib/click-log";
import { plainError, SignInExpiredError, traceGroupTitle } from "./driver-helpers";
import type { FrameEncoder } from "./encoder";
import type { ActionOpts, TakeClick, TakeSink } from "./flow-types";
import { CursorDriver } from "./recorder";
import { resolveTarget, SelectorError } from "./selectors";
import type { OpenSession } from "./session";
import { runStep } from "./take-steps";

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

export type Recording = { encoder: FrameEncoder; record: RecordOptions };

export type TakeOptions = {
  workflow: Workflow;
  chain: WorkflowNode[];
  vars: Record<string, string>;
  projectDir: string;
  executablePath: string;
  live?: LiveSink | null;
  tracePath?: string;
  traceGroups?: string[];
  signIn?: { url: string; name: string };
  storageState?: string | null;
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

const ACTION_KINDS: Partial<Record<string, Action["kind"]>> = {
  "browser.type": "type",
  "browser.click": "click",
  "browser.hover": "hover",
  "browser.drag": "drag",
};

export class TakeRun implements TakeSink {
  readonly actions: Action[] = [];
  readonly keys: KeyEvent[] = [];
  readonly clicks: TakeClick[] = [];
  readonly cursorTrack: CursorSample[] = [];
  readonly hoisted = new Map<string, Locator>();
  cursorDriver: CursorDriver | null = null;
  t0 = Date.now();
  private lastLiveCursorAt = 0;

  constructor(
    readonly opts: TakeOptions,
    readonly session: OpenSession,
    readonly page: Page,
    readonly recording: Recording | null,
  ) {}

  get vars() {
    return this.opts.vars;
  }

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

  pushAction(action: Omit<Action, "nodeId">, node: WorkflowNode) {
    this.actions.push({ ...action, nodeId: node.id });
  }

  start() {
    this.t0 = Date.now();
    this.recording?.encoder.setOrigin(this.t0);
    this.cursorDriver = new CursorDriver(this.page, this.elapsed, this);
  }

  async run(node: WorkflowNode) {
    const { opts } = this;
    throwIfCancelled(opts.signal);
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

  private clickAction(c: TakeClick, node: WorkflowNode): Action {
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
      selectorUsed: c.selector,
      zoom: c.zoom,
      zoomScale: c.zoomScale,
      text: kind === "type" ? c.text : undefined,
      nodeId: node.id,
      ...(c.path ? { path: c.path } : {}),
      ...(c.drop ? { drop: c.drop } : {}),
    };
  }
}
