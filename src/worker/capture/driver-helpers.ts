import { stripVTControlCharacters } from "node:util";
import { parseParams } from "../../shared/nodes/define";
import { waitNode } from "../../shared/nodes/browser";
import type { WorkflowNode } from "../../shared/schema/workflow";
import { interpolate } from "../../shared/vars";
import { errorMessage } from "../../shared/errors";
import { CSS_PREFIX } from "./flow-types";

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

export const withoutCallName = (text: string): string =>
  text.split("\n")[0].replace(/^(locator|page|frame|elementHandle|mouse|keyboard)\.\w+:\s*/, "");

export function plainError(
  error: unknown,
  node: WorkflowNode,
  vars: Record<string, string>,
): string {
  const text = stripVTControlCharacters(errorMessage(error));
  const timeout = /Timeout (\d+)ms exceeded/.exec(text);
  const target = interpolate(node.params, vars).target;
  const what =
    typeof target === "string" && target.trim()
      ? `“${target.startsWith(CSS_PREFIX) ? target.slice(CSS_PREFIX.length) : target}”`
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
  return withoutCallName(text);
}

export function traceGroupTitle(label: string, node: WorkflowNode, vars: Record<string, string>) {
  const p = interpolate(node.params, vars);
  const subject = [p.target, p.url, p.text, p.keys].find(
    (v): v is string => typeof v === "string" && v.trim() !== "",
  );
  return subject ? `${label} · ${subject}` : label;
}

const OFF_CAMERA = new Set([
  "browser.hideElements",
  "browser.setState",
  "browser.mockRoute",
  "browser.freezeClock",
  "browser.hoist",
]);

export function setupLength(steps: WorkflowNode[], vars: Record<string, string>): number {
  const firstNav = steps.findIndex((n) => n.type === "browser.navigate");
  if (firstNav < 0) return 0;
  let end = firstNav + 1;
  for (; end < steps.length; end++) {
    const node = steps[end];
    if (OFF_CAMERA.has(node.type)) continue;
    if (node.type === "browser.wait" && parseParams(waitNode, node, vars).mode !== "ms") continue;
    break;
  }
  return end;
}
