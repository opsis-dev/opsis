import { isNumber, isRecord, isString } from "../guards";
import { settingsInFrames } from "./seconds";
import type { ElementMeta } from "./types";

type Settings = Record<string, unknown>;

type Timing = {
  unit: "frames" | "authored" | "seconds";
  end: (s: Settings, fps: number) => number;
};

export const AUTHORED_FPS = 30;

const CUT_OFF_HOLD = AUTHORED_FPS / 2;

const RELEASE_TEASER_SECONDS = (960 * 1001) / 60000;

const LAUNCH_ANYTHING_SECONDS = 1600 / 60;

const num = (value: unknown, fallback: number): number =>
  isNumber(value) && Number.isFinite(value) ? value : fallback;

const str = (value: unknown): string => (isString(value) ? value : "");

const list = (value: unknown): unknown[] => (Array.isArray(value) ? value : []);

const field = (row: unknown, key: string): unknown => (isRecord(row) ? row[key] : undefined);

const lastIndex = (count: number): number => Math.max(0, count - 1);

const characters = (text: string): number => Array.from(text).length;

const words = (text: string): number => text.trim().split(/\s+/).filter(Boolean).length;

const lines = (text: string): number => text.split("\n").length;

const commaList = (text: string): string[] =>
  text
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean);

const frames = (end: Timing["end"]): Timing => ({ unit: "frames", end });

const seconds = (end: (s: Settings) => number): Timing => ({ unit: "seconds", end });

const authored = (end: (s: Settings) => number): Timing => ({ unit: "authored", end });

function wordPushEnd(text: string, wordGap: number, accel: number): number {
  const count = words(text);
  if (count === 0) return 0;
  const index = count - 1;
  const reveal = accel === 1 ? index * wordGap : (wordGap * (1 - accel ** index)) / (1 - accel);
  return Math.ceil(reveal) + 12;
}

const KEYSTROKE_HOLD = 8;
const KEYSTROKE_EXIT = 8;

function keystrokeEnd(s: Settings): number {
  const linger = num(s.linger, 30);
  const ends = list(s.steps).flatMap((step) => {
    const at = field(step, "at");
    if (!isNumber(at) || !Number.isFinite(at) || !str(field(step, "keys")).trim()) return [];
    return [at + num(field(step, "hold"), KEYSTROKE_HOLD) + linger + KEYSTROKE_EXIT];
  });
  return Math.max(0, ...ends);
}

const TERMINAL_START = 10;
const TERMINAL_DELAY = 8;
const TERMINAL_ELLIPSIS_PAUSE = 18;

function terminalEnd(s: Settings): number {
  const perFrame = Math.max(0.01, num(s.chunkSize, 1) * num(s.charsPerFrame, 1));
  return list(s.lines).reduce<number>((end, line) => {
    const text = str(field(line, "text"));
    const pause = field(line, "pause");
    const autoPause = isNumber(pause)
      ? pause
      : text.trimEnd().endsWith("...")
        ? TERMINAL_ELLIPSIS_PAUSE
        : 0;
    const typing = Math.ceil(text.length / perFrame);
    return end + num(field(line, "delay"), TERMINAL_DELAY) + typing + autoPause;
  }, TERMINAL_START);
}

const MORPH_BASE = 36;
const MORPH_ENTER_START = 20;
const MORPH_RISE = 8;
const MORPH_SHOW = 24;
const MORPH_FALL = 14;

function codeMorphEnd(s: Settings): number {
  const morph = Math.min(120, Math.max(12, num(s.morphDuration, MORPH_BASE)));
  const starts: number[] = [];
  let ready = 0;
  const ats = list(s.steps)
    .map((step) => Math.max(0, num(field(step, "at"), 0)))
    .toSorted((a, b) => a - b);
  for (const at of ats.slice(1)) {
    const start = Math.max(at, ready);
    starts.push(start);
    ready = start + morph;
  }
  const last = starts.at(-1);
  if (last === undefined) return 0;
  const highlight = last + (MORPH_ENTER_START * morph) / MORPH_BASE + MORPH_RISE + MORPH_SHOW;
  return s.highlight === false ? last + morph : Math.max(last + morph, highlight + MORPH_FALL);
}

function handwriteFrames(text: string, perStep: number, step: number): number {
  return Math.ceil(characters(text) / perStep) * step + step;
}

function checkListEnd(s: Settings): number {
  const step = num(s.step, 3);
  const itemGap = num(s.itemGap, step * 6);
  const perStep = num(s.perStep, 1.6);
  const closeGap = num(s.closeGap, step * 3);
  const items = list(s.items).map((item) =>
    isString(item)
      ? { text: item, checked: true }
      : { text: str(field(item, "text")), checked: field(item, "checked") !== false },
  );
  const labelEnds = items.map(
    (item, i) => i * itemGap + step * 2 + handwriteFrames(item.text, perStep, step),
  );
  const enterEnd = Math.max(0, ...labelEnds);
  const checked = items.filter((item) => item.checked).length;
  const strikeEnd = checked ? enterEnd + (checked - 1) * closeGap + step * 4 : 0;
  return Math.max(enterEnd, strikeEnd);
}

const AGENT_PLAN_PER_WORD = 0.72;
const AGENT_ANSWER_PER_WORD = 1;
const AGENT_ANSWER_PER_LINE = 1;

function agentRunEnd(s: Settings): number {
  const perFrame = Math.min(20, Math.max(0.25, num(s.compression, 3))) / AUTHORED_FPS;
  const plan = list(s.runPlan).map(str).join("\n");
  const answer = str(s.runAnswer);
  const think = 16 + 14 + words(plan) * AGENT_PLAN_PER_WORD + 8;
  const tools = list(s.runSteps).reduce<number>(
    (sum, step) =>
      sum + 5 + Math.min(120, Math.max(12, Math.round(num(field(step, "duration"), 0) / perFrame))),
    0,
  );
  const answering = words(answer) * AGENT_ANSWER_PER_WORD + lines(answer) * AGENT_ANSWER_PER_LINE;
  return think + tools + 8 + answering + 16;
}

function logoWallEnd(s: Settings): number {
  const count = list(s.logos).length;
  const columns = Math.max(1, Math.round(num(s.columns, 4)));
  const startAt = num(s.startAtSeconds, 0.32);
  let diagonal = 0;
  for (let index = 0; index < count; index++)
    diagonal = Math.max(diagonal, Math.floor(index / columns) + (index % columns));
  const arrived = startAt + lastIndex(count) * num(s.arriveStaggerSeconds, 0.06) + 0.36;
  const warmed = startAt + 0.3 + diagonal * num(s.staggerSeconds, 0.24) + num(s.warmSeconds, 0.6);
  return Math.max(arrived, warmed);
}

function endCardEnd(s: Settings): number {
  const url = str(s.url);
  return Math.max(
    0.81,
    str(s.subtitle) ? 1.1 : 0,
    str(s.cta) ? 2.75 : 0,
    url ? 1.24 + url.length / 26 : 0,
    list(s.handles).length ? 2.1 : 0,
  );
}

function codeRevealEnd(s: Settings): number {
  const rows = str(s.code).replace(/\s+$/, "").replace(/^\n+/, "").split("\n");
  const typed = rows.reduce((sum, row) => sum + row.trimStart().length, 0);
  return 0.4 + typed / 55 + lastIndex(rows.length) * 0.07 + 0.84;
}

function fileTreeRows(paths: string[]): number {
  const rows = new Set<string>();
  for (const path of paths) {
    const parts = path.split("/").filter(Boolean);
    for (let depth = 1; depth <= parts.length; depth++) rows.add(parts.slice(0, depth).join("/"));
  }
  return rows.size;
}

function notificationEnd(s: Settings): number {
  const startAt = num(s.startAtSeconds, 0.3);
  const stagger = num(s.staggerSeconds, 0.62);
  const life = num(s.lifeSeconds, 2.3);
  return list(s.toasts).reduce<number>((end, toast, index) => {
    const spawn = num(field(toast, "atSeconds"), startAt + stagger * index);
    return Math.max(end, spawn + num(field(toast, "holdFor"), life) + 0.3);
  }, 0);
}

function punchLinesEnd(s: Settings): number {
  const cards = str(s.script)
    .split("|")
    .map((part) => part.trim())
    .filter(Boolean).length;
  const holds = commaList(str(s.holds)).map(Number).filter(Number.isFinite);
  let total = 0;
  for (let i = 0; i < cards; i++)
    total += Math.max(1, holds.length ? holds[Math.min(i, holds.length - 1)] : 48);
  return total;
}

const ELEMENT_TIMINGS: Partial<Record<string, Timing>> = {
  "remocn/soft-blur-in": authored((s) => lastIndex(characters(str(s.text))) + 27),
  "remocn/typewriter": authored(
    (s) => (characters(str(s.text)) / Math.max(0.1, num(s.charsPerSecond, 22))) * AUTHORED_FPS,
  ),
  "remocn/matrix-decode": authored((s) => num(s.revealDuration, 60)),
  "remocn/slot-machine-roll": authored(
    (s) => lastIndex(Math.max(str(s.from).length, str(s.to).length)) * 4,
  ),
  "remocn/staggered-fade-up": authored(
    (s) => lastIndex(str(s.text).split(" ").length) * num(s.staggerDelay, 4) + 12,
  ),
  "remocn/kinetic-center-build": authored(
    (s) => 10 + lastIndex(str(s.text).split(" ").length) * 13,
  ),
  "remocn/mask-reveal-up": authored((s) => lastIndex(lines(str(s.text))) * 5),
  "remocn/line-by-line-slide": authored((s) => lastIndex(lines(str(s.text))) * 6),
  "remocn/word-push": authored((s) =>
    wordPushEnd(str(s.text), num(s.wordGap, 10), num(s.accel, 0.8)),
  ),
  "remocn/rgb-glitch-text": authored((s) => num(s.glitchAt, 20) + num(s.glitchDuration, 8)),
  "remocn/ink-underline": authored((s) => num(s.durationSteps, 5) * num(s.step, 3)),
  "remocn/scribble-circle": authored((s) => num(s.durationSteps, 10) * num(s.step, 3)),
  "remocn/ink-arrow": authored((s) => num(s.drawDur, 36) + num(s.headDur, num(s.step, 3) * 4)),
  "remocn/confetti": authored((s) => num(s.lifetime, 90)),
  "remocn/keystroke": authored(keystrokeEnd),
  "remocn/terminal-simulator": authored(terminalEnd),
  "remocn/glass-code-block": authored(
    (s) => lastIndex(lines(str(s.code))) * num(s.staggerFrames, 4) + 8,
  ),
  "remocn/code-morph": authored(codeMorphEnd),
  "remocn/logo-enter": authored((s) => lastIndex(list(s.logos).length) * num(s.stagger, 7)),
  "remocn/check-list": authored(checkListEnd),
  "remocn/search-reveal": authored((s) => {
    const step = Math.min(12, Math.max(1, num(s.framesPerCharacter, 4)));
    return Math.max(90, 54 + characters(str(s.text)) * step + 10);
  }),
  "remocn/agent-run": authored(agentRunEnd),
  "remocn/signup-flow": authored(() => 348),
  "remocn/command-menu": authored((s) =>
    s.autoplay === false ? 0 : 24 + characters(str(s.query)) * 2.5 + 8 + 18 + 6,
  ),
  "remocn/release-teaser": seconds(() => RELEASE_TEASER_SECONDS),
  "remocn/launch-anything": seconds(() => LAUNCH_ANYTHING_SECONDS),
  "remocn/animated-bar-chart": authored(
    (s) => lastIndex(list(s.bars).length) * num(s.staggerFrames, 6),
  ),
  "remotion-ui/metric-ticker": frames(
    (s) => Math.min(list(s.metrics).length, Math.max(1, num(s.maxCards, 4))) * 8,
  ),
  "remotion-ui/funnel-chart": frames(
    (s) =>
      lastIndex(list(s.stages).length) * num(s.staggerInFrames, 12) + num(s.durationInFrames, 22),
  ),
  "remotion-ui/donut-chart": frames(
    (s) =>
      lastIndex(list(s.segments).length) * num(s.staggerInFrames, 9) + num(s.durationInFrames, 26),
  ),
  "remotion-ui/comparison-bars": frames(
    (s) =>
      lastIndex(list(s.rows).length) * num(s.staggerInFrames, 12) +
      num(s.pairOffsetInFrames, 6) +
      num(s.durationInFrames, 30),
  ),
  "remotion-ui/bar-chart-race": frames((s) => {
    const steps = Math.max(0, ...list(s.series).map((row) => list(field(row, "values")).length));
    return lastIndex(steps) * num(s.framesPerStep, 26);
  }),
  "remotion-ui/gauge-dial": frames((s) => num(s.durationInFrames, 44)),
  "remotion-ui/handwriting-text": frames(
    (s) =>
      lastIndex(characters(str(s.text))) * num(s.staggerInFrames, 3) + num(s.durationInFrames, 10),
  ),
  "remotion-ui/neon-flicker-text": frames(
    (s) => lastIndex(characters(str(s.text))) * num(s.staggerInFrames, 0),
  ),
  "remotion-ui/text-mask-video": frames((s) => num(s.durationInFrames, 0)),
  "remotion-ui/pricing-card": seconds((s) =>
    Math.max(
      num(s.priceAtSeconds, 0.42) + num(s.rollSeconds, 0.9),
      num(s.featuresAtSeconds, 1.05) +
        lastIndex(list(s.features).length) * num(s.featureStaggerSeconds, 0.22) +
        0.58,
    ),
  ),
  "remotion-ui/comparison-table": seconds((s) => {
    const rows = list(s.rows);
    const cells = Math.max(0, ...rows.map((row) => list(field(row, "cells")).length));
    return (
      num(s.startAtSeconds, 0.5) +
      lastIndex(rows.length) * num(s.rowStaggerSeconds, 0.26) +
      Math.max(0.4, 0.39 + lastIndex(cells) * 0.07)
    );
  }),
  "remotion-ui/changelog-entry": seconds((s) =>
    Math.max(
      0.84,
      num(s.startAtSeconds, 0.66) +
        lastIndex(list(s.changes).length) * num(s.staggerSeconds, 0.28) +
        0.46,
    ),
  ),
  "remotion-ui/logo-wall": seconds(logoWallEnd),
  "remotion-ui/end-card": seconds(endCardEnd),
  "remotion-ui/code-reveal": seconds(codeRevealEnd),
  "remotion-ui/code-diff-wipe": seconds((s) => 0.7 + num(s.wipeSeconds, 1.7) + 1.4),
  "remotion-ui/kanban-move": seconds((s) => {
    const cards = list(s.cards);
    const moves = cards.flatMap((card) => {
      const at = field(card, "moveAtSeconds");
      return isNumber(at) && field(card, "moveTo") !== undefined
        ? [at + num(s.moveSeconds, 0.62)]
        : [];
    });
    return Math.max(
      0.18 + lastIndex(cards.length) * num(s.dealStaggerSeconds, 0.08) + 0.42,
      ...moves,
    );
  }),
  "remotion-ui/file-tree-reveal": seconds(
    (s) =>
      0.42 +
      lastIndex(fileTreeRows(list(s.nodes).map(str))) * num(s.rowStagger, 0.18) +
      0.34 +
      (str(s.selectedPath) ? 0.64 : 0),
  ),
  "remotion-ui/commit-graph": seconds(
    (s) =>
      num(s.startAtSeconds, 0.34) +
      Math.max(1, list(s.commits).length - 1) * num(s.stepSeconds, 0.36),
  ),
  "remotion-ui/notification-stack": seconds(notificationEnd),
  "snapcn/punch-lines": authored(punchLinesEnd),
  "snapcn/status-cycle": authored(
    (s) =>
      num(s.startAt, 0) +
      num(s.introFrames, 24) +
      lastIndex(commaList(str(s.statuses)).length) * num(s.statusHold, 30),
  ),
  "snapcn/text-highlight": authored(
    (s) =>
      num(s.startAt, 6) +
      num(s.drawDuration, 14) +
      num(s.holdDuration, 12) +
      num(s.rushDuration, 24),
  ),
  "snapcn/logo-drift": seconds((s) => num(s.exitAt, 3.13) + num(s.exitDuration, 0)),
  "snapcn/laptop-frame": authored(() => 0),
  "snapcn/phone-frame": authored(() => 0),
  "snapcn/screen-recording": authored(() => 0),
  "snapcn/announce-title": authored(() => 0),
  "snapcn/logo-assemble": authored(() => 0),
  "snapcn/hero-launch": authored(() => 0),
};

const OWN_CLOCK = /^(remocn|snapcn)\//;

const SCENE_TEMPLATES: ReadonlySet<string> = new Set([
  "remocn/release-teaser",
  "remocn/launch-anything",
]);

const KEEPS_ITS_LENGTH = authored(() => 0);

function timingOf(meta: ElementMeta): Timing | undefined {
  const timing = ELEMENT_TIMINGS[meta.key];
  if (timing) return timing;
  const ownClock =
    meta.kind === "component" && OWN_CLOCK.test(meta.key) && !SCENE_TEMPLATES.has(meta.key);
  return ownClock ? KEEPS_ITS_LENGTH : undefined;
}

export const fitsItsAnimation = (meta: ElementMeta): boolean => timingOf(meta) !== undefined;

function endFrames(timing: Timing, settings: Settings, fps: number): number {
  const end = timing.end(settings, fps);
  if (timing.unit === "seconds") return end * fps;
  return timing.unit === "authored" ? (end * fps) / AUTHORED_FPS : end;
}

export function animationLength(meta: ElementMeta, params: Settings, fps: number): number {
  const timing = timingOf(meta);
  if (!timing) return meta.defaultDurationInFrames;
  const defaults = { ...settingsInFrames(meta, meta.defaultProps), ...meta.fixedProps };
  const settings = {
    ...settingsInFrames(meta, { ...meta.defaultProps, ...params }),
    ...meta.fixedProps,
  };
  const designedHold =
    meta.defaultDurationInFrames -
    endFrames(timing, defaults, AUTHORED_FPS) / num(defaults.speed, 1);
  const hold = designedHold < 0 ? CUT_OFF_HOLD : designedHold;
  const scaledHold = timing.unit === "frames" ? hold : (hold * fps) / AUTHORED_FPS;
  const speed = num(settings.speed, 1);
  if (speed <= 0) return 1;
  if (timing === KEEPS_ITS_LENGTH) return Math.max(1, Math.ceil(scaledHold / speed));
  return Math.max(1, Math.ceil(endFrames(timing, settings, fps) / speed + scaledHold));
}
