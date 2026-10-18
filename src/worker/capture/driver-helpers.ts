import { stripVTControlCharacters } from "node:util";
import type { WorkflowNode } from "../../shared/schema/workflow";
import { interpolate } from "../../shared/vars";
import { errorMessage, withoutCallName } from "../../shared/errors";
import { round1 } from "../../shared/math";
import { CSS_PREFIX } from "./flow-types";

function samePage(a: string, b: string): boolean {
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
  const text = stripVTControlCharacters(errorMessage(error));
  const timeout = /Timeout (\d+)ms exceeded/.exec(text);
  const target = interpolate(node.params, vars).target;
  const what =
    typeof target === "string" && target.trim()
      ? `“${target.startsWith(CSS_PREFIX) ? target.slice(CSS_PREFIX.length) : target}”`
      : "the element";
  if (timeout) {
    const seconds = round1(Number(timeout[1]) / 1000);
    return /waiting for (locator|getBy)/.test(text)
      ? `Couldn't find ${what} on the page within ${seconds} s.`
      : `The page didn't respond within ${seconds} s.`;
  }
  const network = /net::(ERR_\w+)/.exec(text)?.[1];
  if (network === "ERR_NAME_NOT_RESOLVED") return "Couldn't reach the site. Check the address.";
  if (network) return `Couldn't load the page (${network}).`;
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
