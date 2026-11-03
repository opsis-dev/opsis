import { displayAddress } from "#video/common/format.ts";
import type { ActionLog, AddressMode } from "#video/generated/spec.ts";

const SCHEME = /^[a-z]+:\/\//i;

function pageAt(log: ActionLog, timeMs: number): string | undefined {
  const addresses = log.addresses ?? [];
  const shown = addresses.findLast((a) => a.t <= timeMs) ?? addresses[0];
  return shown?.url;
}

function firstPage(log: ActionLog): string | undefined {
  return log.actions.find((a) => a.kind === "navigate")?.label;
}

function pathOf(url: string): string {
  if (!URL.canParse(url)) return "";
  const { protocol, pathname, hash } = new URL(url);
  return protocol === "http:" || protocol === "https:" ? `${pathname}${hash}` : "";
}

function hostOf(typed: string): string {
  return typed.replace(SCHEME, "").split(/[/?#]/, 1)[0];
}

export function frameAddress(
  log: ActionLog,
  timeMs: number,
  typed?: string,
  mode?: AddressMode,
): string | undefined {
  const address = typed?.trim();
  if (!address) {
    const page = firstPage(log);
    return page ? displayAddress(page) : undefined;
  }
  if (mode !== "host") return displayAddress(address);
  const page = pageAt(log, timeMs) ?? firstPage(log);
  return displayAddress(`${hostOf(address)}${page ? pathOf(page) : ""}`);
}

export function terminalTitle(log: ActionLog, timeMs: number, typed?: string): string | undefined {
  const text = typed?.trim();
  if (text) return text;
  const titles = log.titles ?? [];
  return (titles.findLast((t) => t.t <= timeMs) ?? titles[0])?.title || undefined;
}
