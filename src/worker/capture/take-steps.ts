import path from "node:path";
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
  setStateNode,
  typeNode,
  uploadNode,
  waitNode,
} from "../../shared/nodes/browser";
import { customLabel, nodeName } from "../../shared/nodes/registry";
import type { WorkflowNode } from "../../shared/schema/workflow";
import { PAUSE_MARK } from "../../shared/vars";
import { nonBlankLines } from "../../shared/text-input";
import { parsePath } from "./drag-path";
import { landedOnSignIn, SignInExpiredError } from "./driver-helpers";
import { elementOf, type DragTo } from "./flow-types";
import {
  addStyles,
  beforeFirstLoad,
  CHAT_WIDGET_SELECTORS,
  COOKIE_BANNER_SELECTORS,
  hidingCss,
  runUserScript,
} from "./page-scripts";
import type { CursorDriver } from "./recorder";
import { dumpVisible, type Resolved } from "./selectors";
import type { TakeRun } from "./take-run";

const FIRST_PAGE_MIN_WAIT_MS = 500;
const SCROLL_TICK_MS = 16;

type StepHandler = (take: TakeRun, node: WorkflowNode) => Promise<void>;

async function onTarget(
  take: TakeRun,
  node: WorkflowNode,
  p: { target: string; after: number },
  act: (driver: CursorDriver, target: Resolved, label: string) => Promise<void>,
): Promise<void> {
  const resolved = await take.resolve(p.target);
  await act(take.rolling(), resolved, nodeName(node, p.target));
  await take.rolling().pause(p.after);
}

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
    await onTarget(take, node, p, async (driver, t, label) =>
      driver.moveAndClick(t.target, label, await take.motion(p, t.by)),
    );
  },
  "browser.type": async (take, node) => {
    const p = take.params(typeNode, node);
    await onTarget(take, node, p, async (driver, t, label) => {
      await driver.typeInto(t.target, p.text.split(PAUSE_MARK), label, {
        ...(await take.motion(p, t.by)),
        chunkPauseMs: p.chunkPauseMs,
      });
      if (p.pressEnter) await driver.pressKey("Enter", "Enter");
    });
  },
  "browser.hover": async (take, node) => {
    const p = take.params(hoverNode, node);
    await onTarget(take, node, p, async (driver, t, label) => {
      if (p.zoom) {
        await driver.focus(t.target, label, await take.motion(p, t.by));
        return;
      }
      await driver.moveTo(t.target);
      await driver.pause(p.hoverMs);
    });
  },
  "browser.drag": async (take, node) => {
    const p = take.params(dragNode, node);
    await onTarget(take, node, p, async (driver, from, label) => {
      if (p.mode === "element" && !p.to.trim()) throw new Error("Pick where to drop it.");
      const to: DragTo =
        p.mode === "element"
          ? { target: (await take.resolve(p.to)).target }
          : p.mode === "by"
            ? { by: { x: p.dx, y: p.dy } }
            : { path: parsePath(p.path) };
      await driver.drag(from.target, to, label, {
        ...(await take.motion(p, from.by)),
        holdMs: p.holdMs,
        dragMs: p.dragMs,
        method: p.method,
      });
    });
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
        label: customLabel(node),
      },
      node,
    );
    await take.pauseIfRolling(p.after);
  },
  "browser.select": async (take, node) => {
    const p = take.params(selectNode, node);
    await onTarget(take, node, p, async (driver, t, label) => {
      await driver.focus(t.target, label, await take.motion(p, t.by));
      const loc = elementOf(t.target);
      await loc.selectOption(p.value).catch(async () => loc.selectOption({ label: p.value }));
    });
  },
  "browser.upload": async (take, node) => {
    const p = take.params(uploadNode, node);
    const t = await take.resolve(p.target);
    await elementOf(t.target).setInputFiles(path.resolve(take.opts.projectDir, p.file));
    await take.pauseIfRolling(p.after);
  },
  "browser.press": async (take, node) => {
    const p = take.params(pressNode, node);
    if (take.cursorDriver) await take.cursorDriver.pressKey(p.keys, nodeName(node, p.keys));
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
    await take.page.evaluate(runUserScript(p.script));
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

export function runStep(take: TakeRun, node: WorkflowNode): Promise<void> {
  const handler = STEPS[node.type];
  if (!handler)
    throw new Error(`"${nodeName({ type: node.type })}" can't be part of a browser take.`);
  return handler(take, node);
}
