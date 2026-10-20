import type { Locator, Page } from "playwright-core";
import { errorMessage, throwIfCancelled } from "../../shared/errors";
import { INTERACTIVE_SELECTOR } from "../../shared/interactive";
import { parsePoint } from "../../shared/schema/actions";
import { withoutCallName } from "./driver-helpers";
import { CSS_PREFIX, type Target } from "./flow-types";

type FindPage = Pick<
  Page,
  | "getByRole"
  | "getByLabel"
  | "getByPlaceholder"
  | "getByTestId"
  | "getByText"
  | "locator"
  | "waitForTimeout"
  | "evaluate"
>;

type Candidate = { by: string; locator: Locator };

export type Resolved = { target: Target; by: string };

type ResolveOptions = {
  log: (level: "info", message: string) => void;
  signal: AbortSignal;
  hoisted?: ReadonlyMap<string, Locator>;
  timeoutMs?: number;
};

const DEFAULT_TIMEOUT_MS = 15_000;
const POLL_MS = 250;
const PAGE_CHANGING = /detached|not attached|Execution context was destroyed|navigat/i;
const DUMP_LIMIT = 24;

const escapeRegExp = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

const NAMED_ROLES = ["button", "link", "textbox", "combobox", "option", "tab"] as const;

const ANCESTOR_ROLES = ["button", "link", "menuitem", "tab", "option", "combobox"];
const ANCESTOR_TAGS = ["button", "a"];
const INTERACTIVE_ANCESTOR = `xpath=ancestor-or-self::*[${[
  ...ANCESTOR_ROLES.map((role) => `@role="${role}"`),
  ...ANCESTOR_TAGS.map((tag) => `self::${tag}`),
].join(" or ")}][1]`;

function autoCandidates(page: FindPage, name: string): Candidate[] {
  const exact = new RegExp(`^${escapeRegExp(name)}$`, "i");
  const loose = new RegExp(escapeRegExp(name), "i");
  return [
    ...NAMED_ROLES.map((role) => ({
      by: `role=${role} name="${name}"`,
      locator: page.getByRole(role, { name: exact }),
    })),
    { by: `label="${name}"`, locator: page.getByLabel(exact) },
    { by: `placeholder~"${name}"`, locator: page.getByPlaceholder(loose) },
    { by: `testid="${name}"`, locator: page.getByTestId(name) },
    {
      by: `control containing "${name}"`,
      locator: page.locator(INTERACTIVE_SELECTOR).filter({ hasText: loose }),
    },
    {
      by: `text "${name}" -> control`,
      locator: page.getByText(loose).locator(INTERACTIVE_ANCESTOR),
    },
    { by: `text "${name}"`, locator: page.getByText(loose) },
  ];
}

export class SelectorError extends Error {
  constructor(
    public readonly label: string,
    public readonly tried: string[],
    public readonly dump: string,
  ) {
    super(`Couldn't find “${label}” on the page.\n  Looked for: ${tried.join(" | ")}\n${dump}`);
    this.name = "SelectorError";
  }
}

async function visibleNow(locator: Locator): Promise<boolean> {
  try {
    return await locator.isVisible();
  } catch (error) {
    if (PAGE_CHANGING.test(errorMessage(error))) return false;
    throw error;
  }
}

async function firstVisible(candidates: Candidate[]): Promise<Resolved | null> {
  for (const c of candidates) {
    const locator = c.locator.first();
    if (await visibleNow(locator)) return { target: locator, by: c.by };
  }
  return null;
}

export async function resolveTarget(
  page: FindPage,
  raw: string,
  opts: ResolveOptions,
): Promise<Resolved> {
  const timeoutMs = opts.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  const hit = opts.hoisted?.get(raw);
  if (hit) return { target: hit, by: "hoisted" };
  if (raw.startsWith(CSS_PREFIX)) {
    const locator = page.locator(raw.slice(CSS_PREFIX.length)).first();
    await locator.waitFor({ state: "visible", timeout: timeoutMs });
    return { target: locator, by: raw };
  }
  const point = parsePoint(raw);
  if (point) return { target: point, by: "point" };

  const candidates = autoCandidates(page, raw);
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    throwIfCancelled(opts.signal);
    const found = await firstVisible(candidates);
    if (found) {
      opts.log("info", `✓ Found ${raw}`);
      return found;
    }
    await page.waitForTimeout(POLL_MS);
  }
  throw new SelectorError(
    raw,
    candidates.map((c) => c.by),
    await dumpVisible(page),
  );
}

type OnScreen = {
  tag: string;
  role: string;
  type: string;
  text: string;
  x: number;
  y: number;
  w: number;
  h: number;
};

function listOnScreen({ max, selector }: { max: number; selector: string }): OnScreen[] {
  const shown = (e: Element) => {
    const r = e.getBoundingClientRect();
    const s = getComputedStyle(e);
    return (
      r.width > 2 &&
      r.height > 2 &&
      s.visibility !== "hidden" &&
      s.display !== "none" &&
      s.opacity !== "0" &&
      r.bottom > 0 &&
      r.top < innerHeight &&
      r.right > 0 &&
      r.left < innerWidth
    );
  };
  return Array.from(document.querySelectorAll(selector))
    .filter(shown)
    .slice(0, max)
    .map((e) => {
      const r = e.getBoundingClientRect();
      const text = e.getAttribute("aria-label") || e.getAttribute("placeholder") || e.textContent;
      return {
        tag: e.tagName.toLowerCase(),
        role: e.getAttribute("role") ?? "",
        type: e.getAttribute("type") ?? "",
        text: text.trim().replace(/\s+/g, " ").slice(0, 44),
        x: Math.round(r.x),
        y: Math.round(r.y),
        w: Math.round(r.width),
        h: Math.round(r.height),
      };
    });
}

export async function dumpVisible(page: Pick<Page, "evaluate">): Promise<string> {
  try {
    const items = await page.evaluate(listOnScreen, {
      max: DUMP_LIMIT,
      selector: `${INTERACTIVE_SELECTOR},.ant-select`,
    });
    const lines = items.map(
      (e) =>
        `    · <${e.tag}${e.role ? ` role=${e.role}` : ""}${e.type ? ` type=${e.type}` : ""}> "${e.text}" @${e.x},${e.y} ${e.w}×${e.h}`,
    );
    return `  on screen now:\n${lines.join("\n") || "    (no visible interactive elements)"}`;
  } catch (error) {
    return `  (couldn't list what's on screen: ${withoutCallName(errorMessage(error))})`;
  }
}
