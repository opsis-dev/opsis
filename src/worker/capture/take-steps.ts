import path from "node:path";
import { projectRelPath } from "@opsis/core/asset-ref";
import { pressedChord } from "@opsis/core/chord";
import { parseJsonWith } from "@opsis/core/guards";
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
  signInNode,
  startRecordingNode,
  typeNode,
  uploadNode,
  waitNode,
} from "@opsis/core/nodes/browser";
import type { NodeDef, NodeParams } from "@opsis/core/nodes/define";
import { customLabel, nodeName } from "@opsis/core/nodes/registry";
import type { Action } from "@opsis/core/schema/actions";
import { storageStateSchema } from "@opsis/core/schema/storage-state";
import type { SessionStep } from "@opsis/core/schema/values";
import { secretName } from "@opsis/core/secrets";
import { nonBlankLines } from "@opsis/core/text-input";
import { PAUSE_MARK } from "@opsis/core/vars";
import { wallTimeIn } from "@opsis/core/wall-time";
import type { z } from "zod";
import { isInside } from "../../engine/project-paths";
import { warnFailed } from "../engine/context";
import { parsePath } from "./drag-path";
import { landedOnSignIn } from "./driver-helpers";
import { elementOf, type DragTo } from "./flow-types";
import {
  addStyles,
  beforeFirstLoad,
  CHAT_WIDGET_SELECTORS,
  COOKIE_BANNER_SELECTORS,
  hidingCss,
  invalidSelectors,
  optionValue,
  restoreLocalStorage,
  runUserScript,
  type SavedStorage,
} from "./page-scripts";
import type { CursorDriver } from "./recorder";
import { dumpVisible, type Resolved } from "./selectors";
import type { TakeRun } from "./take-run";

const SCROLL_TICK_MS = 16;

type StepSpec = {
  run: (take: TakeRun, node: SessionStep) => Promise<void>;
  clickKind?: Action["kind"];
};

const step = <S extends z.ZodObject>(
  def: NodeDef<S>,
  run: (take: TakeRun, p: z.infer<S>, node: SessionStep) => Promise<void>,
  more: Omit<StepSpec, "run"> = {},
): [string, StepSpec] => [
  def.type,
  { ...more, run: (take, node) => run(take, def.params.parse(node.params), node) },
];

const isTimeoutError = (error: unknown) => error instanceof Error && error.name === "TimeoutError";

async function onTarget(
  take: TakeRun,
  node: SessionStep,
  p: { target: string; after: number },
  act: (driver: CursorDriver, target: Resolved, label: string) => Promise<void>,
): Promise<void> {
  const resolved = await take.resolve(p.target);
  await act(take.rolling(), resolved, nodeName(node, p.target));
  await take.rolling().pause(p.after);
}

async function dragDestination(take: TakeRun, p: NodeParams<typeof dragNode>): Promise<DragTo> {
  if (p.mode === "by") return { by: { x: p.dx, y: p.dy } };
  if (p.mode === "path") return { path: parsePath(p.path) };
  if (!p.to.trim()) throw new Error("Pick where to drop it.");
  return { target: (await take.resolve(p.to)).target };
}

type ScrollParams = NodeParams<typeof scrollNode>;

async function scrollToElement(take: TakeRun, p: ScrollParams) {
  const loc = elementOf((await take.resolve(p.target)).target);
  if (!p.smooth) return loc.scrollIntoViewIfNeeded();
  await loc.evaluate((el) => el.scrollIntoView({ behavior: "smooth", block: "center" }));
  await take.page.waitForTimeout(p.durationMs);
}

async function scrollByAmount({ page }: TakeRun, p: ScrollParams) {
  if (!p.smooth) return page.evaluate((dy) => window.scrollBy(0, dy), p.deltaY);
  const ticks = Math.max(1, Math.round(p.durationMs / SCROLL_TICK_MS));
  for (let i = 0; i < ticks; i++) {
    await page.mouse.wheel(0, p.deltaY / ticks);
    await page.waitForTimeout(SCROLL_TICK_MS);
  }
}

const STEPS = new Map<string, StepSpec>([
  step(navigateNode, async (take, p, node) => {
    const t = take.elapsed();
    await take.page.goto(p.url, { waitUntil: p.waitUntil });
    take.pushAction(
      { id: `${node.id}:nav`, kind: "navigate", t, tEnd: take.elapsed(), label: p.url },
      node,
    );
    await take.settleAfterPage(p.after);
    const { signedIn } = take;
    if (signedIn && landedOnSignIn(p.url, take.page.url(), signedIn.url))
      throw new Error(`Signed out during the take. Run “${signedIn.name}” again.`);
  }),
  step(
    clickNode,
    (take, p, node) =>
      onTarget(take, node, p, async (driver, t, label) =>
        driver.moveAndClick(t.target, label, await take.motion(p, t.by)),
      ),
    { clickKind: "click" },
  ),
  step(
    typeNode,
    (take, p, node) =>
      onTarget(take, node, p, async (driver, t, label) => {
        await driver.typeInto(t.target, p.text.split(PAUSE_MARK), label, {
          ...(await take.motion(p, t.by)),
          chunkPauseMs: p.chunkPauseMs,
        });
        if (p.pressEnter) await driver.pressKey("Enter", "Enter");
      }),
    { clickKind: "type" },
  ),
  step(
    hoverNode,
    (take, p, node) =>
      onTarget(take, node, p, async (driver, t, label) =>
        driver.focus(t.target, label, await take.motion(p, t.by)),
      ),
    { clickKind: "hover" },
  ),
  step(
    dragNode,
    (take, p, node) =>
      onTarget(take, node, p, async (driver, from, label) => {
        const to = await dragDestination(take, p);
        await driver.drag(from.target, to, label, {
          ...(await take.motion(p, from.by)),
          holdMs: p.holdMs,
          dragMs: p.dragMs,
          method: p.method,
        });
      }),
    { clickKind: "drag" },
  ),
  step(scrollNode, async (take, p, node) => {
    const t = take.elapsed();
    if (p.mode === "element") await scrollToElement(take, p);
    else await scrollByAmount(take, p);
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
  }),
  step(
    selectNode,
    (take, p, node) =>
      onTarget(take, node, p, async (driver, t, label) => {
        await driver.focus(t.target, label, await take.motion(p, t.by));
        const loc = elementOf(t.target);
        const option = await loc.evaluate(optionValue, p.value);
        if (option === undefined) throw new Error(`“${p.value}” isn't an option in “${label}”.`);
        await loc.selectOption(option ?? p.value);
      }),
    { clickKind: "focus" },
  ),
  step(uploadNode, async (take, p) => {
    const t = await take.resolve(p.target);
    const file = path.resolve(take.opts.projectDir, projectRelPath(p.file) ?? p.file);
    if (!isInside(take.opts.projectDir, file))
      throw new Error("Choose a file inside the project folder.");
    await elementOf(t.target).setInputFiles(file);
    await take.pauseIfRolling(p.after);
  }),
  step(pressNode, async (take, p, node) => {
    const chord = pressedChord(p.keys);
    await take.pressKey(chord, nodeName(node, p.keys));
    await take.pauseIfRolling(p.after);
  }),
  step(waitNode, async (take, p, node) => {
    const { page } = take;
    const t = take.elapsed();
    if (p.mode === "element") await take.resolve(p.target, p.timeoutMs);
    else if (p.mode === "network-idle")
      await page.waitForLoadState("networkidle", { timeout: p.timeoutMs });
    else if (p.mode === "url") await page.waitForURL(p.url, { timeout: p.timeoutMs });
    else await page.waitForTimeout(p.ms);
    take.pushAction({ id: `${node.id}:wait`, kind: "wait", t, tEnd: take.elapsed() }, node);
  }),
  step(assertNode, async (take, p) => {
    const scope = p.target ? elementOf((await take.resolve(p.target)).target) : take.page;
    try {
      await scope.getByText(p.text).first().waitFor({ state: "visible", timeout: p.timeoutMs });
    } catch (error) {
      if (!isTimeoutError(error)) throw error;
      throw new Error(`“${p.text}” isn't on the page.\n${await dumpVisible(take.page)}`, {
        cause: error,
      });
    }
  }),
  step(hoistNode, async (take, p) => {
    const t = await take.resolve(p.target);
    take.hoisted.set(p.target, elementOf(t.target));
  }),
  step(setStateNode, async (take, p, node) => {
    if (take.page.url() === "about:blank") {
      await take.session.context.addInitScript({ content: beforeFirstLoad(node.id, p.script) });
      return;
    }
    await take.page.evaluate(runUserScript(p.script));
    if (p.reload) await take.page.reload();
  }),
  step(mockRouteNode, async (take, p) => {
    await take.page.route(p.url, (route) =>
      route.fulfill({ status: p.status, contentType: p.contentType, body: p.body }),
    );
  }),
  step(freezeClockNode, async (take, p) => {
    const zone = await take.page.evaluate(() => Intl.DateTimeFormat().resolvedOptions().timeZone);
    const time = wallTimeIn(p.time, zone);
    if (p.mode === "fixed") await take.page.clock.setFixedTime(time);
    else {
      await take.page.clock.install({ time });
      await take.page.clock.pauseAt(time);
    }
  }),
  step(hideElementsNode, async (take, p) => {
    const selectors = [
      ...(p.cookieBanners ? COOKIE_BANNER_SELECTORS : []),
      ...(p.chatWidgets ? CHAT_WIDGET_SELECTORS : []),
      ...nonBlankLines(p.selectors),
    ];
    if (!selectors.length) return;
    const invalid = await take.page
      .evaluate(invalidSelectors, nonBlankLines(p.selectors))
      .catch(() => []);
    if (invalid.length)
      take.opts.log(
        "warn",
        `Can't hide ${invalid.map((s) => `“${s}”`).join(", ")}: not a CSS selector. The rest are hidden.`,
      );
    const css = hidingCss(selectors);
    await take.session.context.addInitScript(addStyles, css);
    await take.page
      .evaluate(addStyles, css)
      .catch((error: unknown) =>
        warnFailed(
          take.opts.log,
          "Couldn't hide the elements on the current page, only on the next ones",
          error,
        ),
      );
  }),
  step(chapterNode, async (take, p, node) => {
    const description = p.description || undefined;
    take.pushAction(
      {
        id: `${node.id}:chapter`,
        kind: "chapter",
        t: take.elapsed(),
        label: p.title,
        text: description,
      },
      node,
    );
    if (p.showInVideo && take.recording) {
      await take.page.screencast.showChapter(p.title, {
        description,
        duration: p.durationMs,
      });
      await take.page.waitForTimeout(p.durationMs);
    }
  }),
  step(overlayHtmlNode, async (take, p) => {
    if (!take.recording) return;
    const overlay = await take.page.screencast.showOverlay(p.html);
    if (p.durationMs > 0)
      setTimeout(() => void overlay.dispose().catch(() => undefined), p.durationMs);
  }),
  step(signInNode, async (take, p, node) => {
    const name = secretName(p.secretName);
    const saved = await take.opts.secrets.read(name);
    if (!saved)
      throw new Error(`The sign-in “${name}” isn't saved yet. Run “${nodeName(node)}” first.`);
    const state = parseJsonWith(storageStateSchema, saved);
    if (!state?.success)
      throw new Error(`The saved sign-in can't be read. Run “${nodeName(node)}” again.`);
    const { context } = take.session;
    await context.addCookies(state.data.cookies);
    const pageOrigin = URL.parse(take.page.url())?.origin;
    for (const { origin, localStorage } of state.data.origins) {
      const storage: SavedStorage = {
        key: `opsis-sign-in:${node.id}`,
        origin,
        items: localStorage,
      };
      await context.addInitScript(restoreLocalStorage, storage);
      if (origin === pageOrigin) await take.page.evaluate(restoreLocalStorage, storage);
    }
    take.signedIn = { url: p.url, name: nodeName(node) };
  }),
  step(startRecordingNode, async () => undefined),
]);

export async function runStep(take: TakeRun, node: SessionStep): Promise<void> {
  const spec = STEPS.get(node.type);
  if (!spec) throw new Error(`"${nodeName({ type: node.type })}" can't be part of a browser take.`);
  await spec.run(take, node);
}

export function clickKindOf(type: string): Action["kind"] {
  const kind = STEPS.get(type)?.clickKind;
  if (!kind) throw new Error(`"${nodeName({ type })}" doesn't point at anything.`);
  return kind;
}

export const videoStart = (steps: readonly SessionStep[]): number =>
  Math.max(
    0,
    steps.findIndex((s) => s.type === startRecordingNode.type),
  );
