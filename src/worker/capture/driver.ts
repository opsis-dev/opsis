import path from "node:path";
import { stripVTControlCharacters } from "node:util";
import type { Locator, Page } from "playwright-core";
import type { z } from "zod";
import type {
  assertNode,
  chapterNode,
  clickNode,
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
  setStateNode,
  typeNode,
  uploadNode,
  waitNode,
} from "../../shared/nodes/browser";
import { getNodeDef } from "../../shared/nodes/registry";
import type { LiveEvent } from "../../shared/live";
import type { Action, ActionLog } from "../../shared/schema/actions";
import type { Workflow, WorkflowNode } from "../../shared/schema/workflow";
import { interpolate } from "../../shared/vars";
import type { CursorSample, KeyEvent } from "../../video/camera/lib/click-log";
import type { ClickEvent, FlowContext, Target } from "./flow-types";
import { buildContext, CURSOR_INIT_SCRIPT } from "./recorder";
import { autoCandidates, dumpVisible, SelectorError } from "./selectors";
import {
  CHAT_WIDGET_SELECTORS,
  COOKIE_BANNER_SELECTORS,
  openSession,
  type SessionParams,
} from "./session";
import type { FrameEncoder } from "./encoder";

export type TakeMode = "record" | "still" | "check";

export type LiveSink = {
  frame: (jpeg: Buffer, width: number, height: number, t: number) => void;
  event: (event: LiveEvent) => void;
};

export type TakeOptions = {
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
  record?: {
    fps: number;
    quality: number;
    establishMs: number;
    endTailMs: number;
    cursor: "synthetic" | "baked" | "none";
    showActions: boolean;
  };
  onPage?: (page: Page) => Promise<void>;
  storageState?: string | null;
  signal: AbortSignal;
  log: (level: "info" | "warn" | "error", message: string) => void;
  step: (nodeId: string, status: "start" | "done" | "error", message?: string) => void;
};

export type TakeResult = {
  log: ActionLog;
  t0: number;
  physicalViewport: { width: number; height: number };
};

type Params<T extends { params: z.ZodType }> = z.infer<T["params"]>;

const CHUNK_SEPARATOR = "¦";

function parsed<T>(node: WorkflowNode, vars: Record<string, string>): T {
  const def = getNodeDef(node.type);
  if (!def) throw new Error(`Unknown node type ${node.type}`);
  return def.params.parse(interpolate(node.params ?? {}, vars)) as T;
}

const nodeLabel = (node: WorkflowNode) => node.label?.trim() ?? "";

export class SignInExpiredError extends Error {
  constructor(readonly secretName: string) {
    super(`The saved sign-in “${secretName}” has expired.`);
    this.name = "SignInExpiredError";
  }
}

export function samePage(a: string, b: string): boolean {
  try {
    const norm = (u: string) => {
      const url = new URL(u);
      return `${url.origin}${url.pathname.replace(/\/(index\.html?)?$/, "")}`;
    };
    return norm(a) === norm(b);
  } catch {
    return false;
  }
}

export function landedOnSignIn(requested: string, current: string, signInUrl: string): boolean {
  return !samePage(requested, signInUrl) && samePage(current, signInUrl);
}

export function plainError(
  error: unknown,
  node: WorkflowNode,
  vars: Record<string, string>,
): string {
  const text = stripVTControlCharacters(error instanceof Error ? error.message : String(error));
  const timeout = /Timeout (\d+)ms exceeded/.exec(text);
  const target = (interpolate(node.params ?? {}, vars) as { target?: unknown }).target;
  const what =
    typeof target === "string" && target.trim()
      ? `“${target.replace(/^css=/, "")}”`
      : "the element";
  if (timeout) {
    const seconds = Math.round(Number(timeout[1]) / 100) / 10;
    return /waiting for (locator|getBy)/.test(text)
      ? `Couldn't find ${what} on the page within ${seconds} s.`
      : `The page didn't respond within ${seconds} s.`;
  }
  if (/net::ERR_NAME_NOT_RESOLVED/.test(text)) return "Couldn't reach the site. Check the address.";
  if (/net::ERR_/.test(text))
    return `Couldn't load the page (${/net::(ERR_\w+)/.exec(text)?.[1]}).`;
  if (/Target (page|context|browser).*closed/.test(text))
    return "The browser closed before the action finished.";
  return text
    .split("\n")[0]
    .replace(/^(locator|page|frame|elementHandle|mouse|keyboard)\.\w+:\s*/, "");
}

export function traceGroupTitle(label: string, node: WorkflowNode, vars: Record<string, string>) {
  const p = interpolate(node.params ?? {}, vars) as Record<string, unknown>;
  const subject = [p.target, p.url, p.text, p.keys].find(
    (v): v is string => typeof v === "string" && v.trim() !== "",
  );
  return subject ? `${label} · ${subject}` : label;
}

export async function runTake(opts: TakeOptions): Promise<TakeResult> {
  const { chain, vars, mode, signal } = opts;
  const head = chain[0];
  if (head?.type !== "browser.session")
    throw new Error("A take must start with an Open browser node.");
  const sessionParams = parsed<SessionParams>(head, vars);
  const steps = chain.slice(1).filter((n) => !n.disabled);

  const live = opts.live ?? null;
  const label = (n: WorkflowNode) => n.label || getNodeDef(n.type)?.title || n.type;
  opts.step(head.id, "start");
  const session = await openSession(sessionParams, {
    storageState: opts.storageState,
    executablePath: opts.executablePath,
  });
  const onAbort = () => void session.close();
  signal.addEventListener("abort", onAbort, { once: true });

  const recording = mode === "record" && !!opts.encoder && !!opts.record;
  if (opts.tracePath)
    await session.context.tracing.start({
      screenshots: !recording,
      snapshots: true,
      sources: false,
    });
  const page = await session.context.newPage();
  if (recording && opts.record!.cursor === "baked")
    await session.context.addInitScript({ content: CURSOR_INIT_SCRIPT });
  opts.step(head.id, "done");

  live?.event({
    kind: "start",
    title: label(head),
    viewport: session.physicalViewport,
    steps: steps.map((n) => ({ nodeId: n.id, label: label(n) })),
  });
  const actions: Action[] = [];
  const clicks = observed<ClickEvent>((c) => {
    live?.event({ kind: "click", x: c.x, y: c.y });
    if (c.rect)
      live?.event({ kind: "target", rect: c.rect, label: c.label ?? "", selector: lastSelector });
  });
  const keys: KeyEvent[] = [];
  let lastCursorAt = 0;
  const cursor = observed<CursorSample>((c) => {
    const now = Date.now();
    if (now - lastCursorAt < 30) return;
    lastCursorAt = now;
    live?.event({ kind: "cursor", x: c.x, y: c.y });
  });
  let lastSelector: string | undefined;
  let lastTyped: string | undefined;
  let failed = false;
  const hoisted = new Map<string, Locator>();
  let t0 = Date.now();
  let takeDurationMs = 0;
  let ctx: FlowContext | null = null;
  const elapsed = () => Date.now() - t0;

  const screencasting = recording || !!live;
  try {
    if (screencasting) {
      await page.screencast.start({
        size: session.physicalViewport,
        quality: opts.record?.quality ?? 80,
        onFrame: async (f) => {
          live?.frame(f.data, f.viewportWidth, f.viewportHeight, f.timestamp);
          await opts.encoder?.push(f.data, f.timestamp);
        },
      });
      if (recording && opts.record!.showActions)
        await page.screencast.showActions({ cursor: "pointer", duration: 600 });
    }

    const firstNav = steps.findIndex((n) => n.type === "browser.navigate");
    const setup = firstNav >= 0 ? steps.slice(0, firstNav + 1) : [];
    const main = firstNav >= 0 ? steps.slice(firstNav + 1) : steps;

    const run = async (node: WorkflowNode) => {
      if (signal.aborted) throw new Error("Cancelled");
      opts.step(node.id, "start");
      live?.event({ kind: "step", nodeId: node.id, status: "running" });
      const clicksBefore = clicks.length;
      const keysBefore = keys.length;
      if (opts.tracePath) {
        await session.context.tracing.group(traceGroupTitle(label(node), node, vars));
        opts.traceGroups?.push(node.id);
      }
      try {
        await runStep(node);
      } catch (raw) {
        const error =
          raw instanceof SelectorError || raw instanceof SignInExpiredError
            ? raw
            : new Error(plainError(raw, node, vars));
        const message = error.message;
        opts.step(node.id, "error", message);
        live?.event({ kind: "step", nodeId: node.id, status: "error", message });
        throw error;
      } finally {
        if (opts.tracePath) await session.context.tracing.groupEnd().catch(() => undefined);
      }
      for (const c of clicks.slice(clicksBefore)) actions.push(clickToAction(c, node));
      for (const k of keys.slice(keysBefore)) {
        actions.push({
          id: `${node.id}:k${actions.length}`,
          kind: "key",
          t: k.tMs,
          text: k.chord,
          label: k.label,
          nodeId: node.id,
        });
      }
      opts.step(node.id, "done");
      live?.event({ kind: "step", nodeId: node.id, status: "done" });
    };

    const runStep = async (node: WorkflowNode) => {
      switch (node.type) {
        case "browser.navigate": {
          const p = parsed<Params<typeof navigateNode>>(node, vars);
          const t = elapsed();
          await page.goto(p.url, { waitUntil: p.waitUntil });
          actions.push({
            id: `${node.id}:nav`,
            kind: "navigate",
            t,
            tEnd: elapsed(),
            label: p.url,
            nodeId: node.id,
          });
          if (ctx) await ctx.pause(p.after);
          else await page.waitForTimeout(500);
          if (opts.signIn && landedOnSignIn(p.url, page.url(), opts.signIn.url))
            throw new SignInExpiredError(opts.signIn.name);
          return;
        }
        case "browser.click": {
          const p = parsed<Params<typeof clickNode>>(node, vars);
          const t = await resolveTarget(page, p.target, hoisted, opts);
          await need(ctx).moveAndClick(t.target, nodeLabel(node) || p.target, {
            cluster: p.cluster || undefined,
            zoom: p.zoom,
            frame: p.frame ? (await resolveTarget(page, p.frame, hoisted, opts)).target : undefined,
            zoomScale: p.zoomScale || undefined,
            travelMs: p.travelMs || undefined,
            hoverMs: p.hoverMs,
          });
          lastSelector = t.by;
          await need(ctx).pause(p.after);
          return;
        }
        case "browser.type": {
          const p = parsed<Params<typeof typeNode>>(node, vars);
          const t = await resolveTarget(page, p.target, hoisted, opts);
          const chunks = p.text.split(CHUNK_SEPARATOR);
          await need(ctx).typeInto(t.target, chunks, nodeLabel(node) || p.target, {
            cluster: p.cluster || undefined,
            zoom: p.zoom,
            frame: p.frame ? (await resolveTarget(page, p.frame, hoisted, opts)).target : undefined,
            chunkPauseMs: p.chunkPauseMs,
          });
          lastSelector = t.by;
          lastTyped = chunks.join("");
          if (p.pressEnter) await need(ctx).pressKey("Enter", "Enter");
          await need(ctx).pause(p.after);
          return;
        }
        case "browser.hover": {
          const p = parsed<Params<typeof hoverNode>>(node, vars);
          const t = await resolveTarget(page, p.target, hoisted, opts);
          if (p.zoom) {
            await need(ctx).focus(t.target, nodeLabel(node) || p.target, {
              cluster: p.cluster || undefined,
              frame: p.frame
                ? (await resolveTarget(page, p.frame, hoisted, opts)).target
                : undefined,
              zoomScale: p.zoomScale || undefined,
              travelMs: p.travelMs || undefined,
              hoverMs: p.hoverMs,
            });
          } else {
            await need(ctx).moveTo(t.target);
            await need(ctx).pause(p.hoverMs);
          }
          lastSelector = t.by;
          await need(ctx).pause(p.after);
          return;
        }
        case "browser.scroll": {
          const p = parsed<Params<typeof scrollNode>>(node, vars);
          const t = elapsed();
          if (p.mode === "element") {
            const target = await resolveTarget(page, p.target, hoisted, opts);
            const loc = asLocator(page, target.target);
            if (p.smooth) {
              await loc.evaluate((el) =>
                el.scrollIntoView({ behavior: "smooth", block: "center" }),
              );
              await page.waitForTimeout(p.durationMs);
            } else await loc.scrollIntoViewIfNeeded();
          } else if (p.smooth) {
            const ticks = Math.max(1, Math.round(p.durationMs / 16));
            for (let i = 0; i < ticks; i++) {
              await page.mouse.wheel(0, p.deltaY / ticks);
              await page.waitForTimeout(16);
            }
          } else {
            await page.evaluate((dy) => window.scrollBy(0, dy), p.deltaY);
          }
          actions.push({
            id: `${node.id}:scroll`,
            kind: "scroll",
            t,
            tEnd: elapsed(),
            label: nodeLabel(node) || undefined,
            nodeId: node.id,
          });
          if (ctx) await ctx.pause(p.after);
          return;
        }
        case "browser.select": {
          const p = parsed<Params<typeof selectNode>>(node, vars);
          const t = await resolveTarget(page, p.target, hoisted, opts);
          await need(ctx).focus(t.target, nodeLabel(node) || p.target, {
            cluster: p.cluster || undefined,
            zoom: p.zoom,
          });
          await asLocator(page, t.target)
            .selectOption(p.value)
            .catch(async () => asLocator(page, t.target).selectOption({ label: p.value }));
          lastSelector = t.by;
          await need(ctx).pause(p.after);
          return;
        }
        case "browser.upload": {
          const p = parsed<Params<typeof uploadNode>>(node, vars);
          const t = await resolveTarget(page, p.target, hoisted, opts);
          await asLocator(page, t.target).setInputFiles(path.resolve(opts.projectDir, p.file));
          if (ctx) await ctx.pause(p.after);
          return;
        }
        case "browser.press": {
          const p = parsed<Params<typeof pressNode>>(node, vars);
          if (ctx) await ctx.pressKey(p.keys, nodeLabel(node) || p.keys);
          else await page.keyboard.press(p.keys);
          if (ctx) await ctx.pause(p.after);
          return;
        }
        case "browser.wait": {
          const p = parsed<Params<typeof waitNode>>(node, vars);
          const t = elapsed();
          if (p.mode === "element") await resolveTarget(page, p.target, hoisted, opts, p.timeoutMs);
          else if (p.mode === "network-idle")
            await page.waitForLoadState("networkidle", { timeout: p.timeoutMs });
          else if (p.mode === "url") await page.waitForURL(p.url, { timeout: p.timeoutMs });
          else await page.waitForTimeout(p.ms);
          actions.push({
            id: `${node.id}:wait`,
            kind: "wait",
            t,
            tEnd: elapsed(),
            nodeId: node.id,
          });
          return;
        }
        case "browser.assert": {
          const p = parsed<Params<typeof assertNode>>(node, vars);
          const scope = p.target
            ? asLocator(page, (await resolveTarget(page, p.target, hoisted, opts)).target)
            : page;
          try {
            await scope
              .getByText(p.text)
              .first()
              .waitFor({ state: "visible", timeout: p.timeoutMs });
          } catch {
            throw new Error(`"${p.text}" isn't on the page.\n${await dumpVisible(page)}`);
          }
          return;
        }
        case "browser.hoist": {
          const p = parsed<Params<typeof hoistNode>>(node, vars);
          const t = await resolveTarget(page, p.target, hoisted, opts);
          hoisted.set(p.target, asLocator(page, t.target));
          return;
        }
        case "browser.setState": {
          const p = parsed<Params<typeof setStateNode>>(node, vars);
          await page.evaluate(`(async () => { ${p.script}\n })()`);
          if (p.reload) await page.reload();
          return;
        }
        case "browser.mockRoute": {
          const p = parsed<Params<typeof mockRouteNode>>(node, vars);
          await page.route(p.url, (route) =>
            route.fulfill({ status: p.status, contentType: p.contentType, body: p.body }),
          );
          return;
        }
        case "browser.freezeClock": {
          const p = parsed<Params<typeof freezeClockNode>>(node, vars);
          const time = new Date(p.time);
          if (p.mode === "fixed") await page.clock.setFixedTime(time);
          else {
            await page.clock.install({ time });
            await page.clock.pauseAt(time);
          }
          return;
        }
        case "browser.hideElements": {
          const p = parsed<Params<typeof hideElementsNode>>(node, vars);
          const selectors = [
            ...(p.cookieBanners ? COOKIE_BANNER_SELECTORS : []),
            ...(p.chatWidgets ? CHAT_WIDGET_SELECTORS : []),
            ...p.selectors
              .split("\n")
              .map((s) => s.trim())
              .filter(Boolean),
          ];
          if (selectors.length) {
            const css = `${selectors.join(",\n")} { display: none !important; visibility: hidden !important; }`;
            const script = `(() => { const s = document.createElement('style'); s.textContent = ${JSON.stringify(css)}; (document.head || document.documentElement).appendChild(s); })()`;
            await session.context.addInitScript({ content: script });
            await page.evaluate(script).catch(() => undefined);
          }
          return;
        }
        case "browser.chapter": {
          const p = parsed<Params<typeof chapterNode>>(node, vars);
          actions.push({
            id: `${node.id}:chapter`,
            kind: "chapter",
            t: elapsed(),
            label: p.title,
            text: p.description || undefined,
            nodeId: node.id,
          });
          if (p.showInVideo && recording) {
            await page.screencast.showChapter(p.title, {
              description: p.description || undefined,
              duration: p.durationMs,
            });
            await page.waitForTimeout(p.durationMs);
          }
          return;
        }
        case "browser.overlayHtml": {
          const p = parsed<Params<typeof overlayHtmlNode>>(node, vars);
          if (recording) await page.screencast.showOverlay(p.html, { duration: p.durationMs });
          return;
        }
        case "browser.session":
          throw new Error("Only one Open browser node is allowed per take.");
        default:
          throw new Error(
            `"${getNodeDef(node.type)?.title ?? node.type}" can't be part of a browser take.`,
          );
      }
    };

    const clickToAction = (c: ClickEvent, node: WorkflowNode): Action => {
      const kind: Action["kind"] =
        node.type === "browser.type"
          ? "type"
          : node.type === "browser.click"
            ? "click"
            : node.type === "browser.hover"
              ? "hover"
              : "focus";
      const a: Action = {
        id: `${node.id}:${actions.length}`,
        kind,
        t: c.tMs,
        tEnd: c.typeEndMs,
        tDepart: c.tDepartMs,
        tDown: c.tDownMs,
        point: { x: c.x, y: c.y },
        rect: c.rect,
        cluster: c.cluster,
        label: c.label,
        selectorUsed: lastSelector,
        zoom: c.zoom,
        zoomScale: c.zoomScale,
        text: kind === "type" ? lastTyped : undefined,
        nodeId: node.id,
      };
      return a;
    };

    for (const node of setup) await run(node);

    t0 = Date.now();
    opts.encoder?.setOrigin(t0);
    ctx = buildContext(page, page.url(), clicks, elapsed, cursor, undefined, keys);
    if (recording) await page.waitForTimeout(opts.record!.establishMs);

    for (const node of main) await run(node);

    if (recording) await page.waitForTimeout(opts.record!.endTailMs);
    else if (mode === "still" && opts.onPage) await opts.onPage(page);
  } catch (error) {
    failed = true;
    if (error instanceof SelectorError) opts.log("error", error.message);
    live?.event({
      kind: "end",
      ok: false,
      message: error instanceof Error ? error.message : String(error),
    });
    throw error;
  } finally {
    const end = Date.now();
    takeDurationMs = end - t0;
    if (screencasting)
      await page.screencast
        .stop()
        .catch((e: unknown) => opts.log("warn", `The recording didn't stop cleanly: ${String(e)}`));
    if (opts.encoder) {
      if (signal.aborted) await opts.encoder.abort();
      else {
        const finished = opts.encoder.finish(end);
        if (failed) await finished.catch(() => undefined);
        else takeDurationMs = await finished;
      }
    }
    if (opts.tracePath)
      await session.context.tracing
        .stop({ path: opts.tracePath })
        .catch((e: unknown) => opts.log("warn", `Couldn't save the trace: ${String(e)}`));
    signal.removeEventListener("abort", onAbort);
    await session.close();
  }
  live?.event({ kind: "end", ok: true });

  return {
    t0,
    physicalViewport: session.physicalViewport,
    log: {
      viewport: session.physicalViewport,
      dpr: sessionParams.dpr,
      t0,
      durationMs: takeDurationMs,
      actions,
      cursorTrack: opts.record?.cursor === "synthetic" ? [...cursor] : undefined,
    },
  };
}

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

function need(ctx: FlowContext | null): FlowContext {
  if (!ctx) throw new Error("Mouse actions must come after the first Go to page action.");
  return ctx;
}

function asLocator(page: Page, target: Target): Locator {
  if (typeof target === "string") return page.getByText(target).first();
  if ("css" in (target as object)) return page.locator((target as { css: string }).css).first();
  if ("x" in (target as object))
    throw new Error("A point on the page isn't an element. Pick an element instead.");
  return target as Locator;
}

async function resolveTarget(
  page: Page,
  raw: string,
  hoisted: Map<string, Locator>,
  opts: Pick<TakeOptions, "log" | "signal">,
  timeoutMs = 15_000,
): Promise<{ target: Target; by: string }> {
  const hit = hoisted.get(raw);
  if (hit) return { target: hit, by: "hoisted" };
  if (raw.startsWith("css=")) {
    const loc = page.locator(raw.slice(4)).first();
    await loc.waitFor({ state: "visible", timeout: timeoutMs });
    return { target: loc, by: raw };
  }
  const point = /^\s*(\d+(?:\.\d+)?)\s*,\s*(\d+(?:\.\d+)?)\s*$/.exec(raw);
  if (point) return { target: { x: Number(point[1]), y: Number(point[2]) }, by: "point" };

  const candidates = autoCandidates(page, raw);
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (opts.signal.aborted) throw new Error("Cancelled");
    for (const c of candidates) {
      try {
        const loc = c.locator.first();
        if (await loc.isVisible()) {
          opts.log("info", `✓ Found ${raw}`);
          return { target: loc, by: c.by };
        }
      } catch {}
    }
    await page.waitForTimeout(250);
  }
  throw new SelectorError(
    raw,
    candidates.map((c) => c.by),
    await dumpVisible(page),
  );
}
