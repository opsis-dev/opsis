import { stripVTControlCharacters } from "node:util";
import { errorMessage } from "../../shared/errors";
import { formatDuration } from "../../shared/format";
import { nonBlankString } from "../../shared/guards";
import type { SessionStep } from "../../shared/schema/values";
import { CSS_PREFIX } from "./flow-types";

export const withoutCallName = (text: string): string =>
  text.split("\n")[0].replace(/^(locator|page|frame|elementHandle|mouse|keyboard)\.\w+:\s*/, "");

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

export function plainError(error: unknown, node: Pick<SessionStep, "params">): string {
  const text = stripVTControlCharacters(errorMessage(error));
  const timeout = /Timeout (\d+)ms exceeded/.exec(text);
  const target = nonBlankString(node.params.target);
  const what = target
    ? `“${target.startsWith(CSS_PREFIX) ? target.slice(CSS_PREFIX.length) : target}”`
    : "the element";
  if (timeout) {
    const wait = formatDuration(Number(timeout[1]));
    return /waiting for (locator|getBy)/.test(text)
      ? `Couldn't find ${what} on the page within ${wait}.`
      : `The page didn't respond within ${wait}.`;
  }
  const network = /net::(ERR_\w+)/.exec(text)?.[1];
  if (network === "ERR_NAME_NOT_RESOLVED") return "Couldn't reach the site. Check the address.";
  if (network) return `Couldn't load the page (${network}).`;
  if (/Target (page|context|browser).*closed/.test(text))
    return "The browser closed before the action finished.";
  return withoutCallName(text);
}

export function traceGroupTitle(label: string, node: Pick<SessionStep, "params">) {
  const p = node.params;
  const subject = [p.target, p.url, p.text, p.keys].map(nonBlankString).find(Boolean);
  return subject ? `${label} · ${subject}` : label;
}
