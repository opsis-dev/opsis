import { displayAddress } from "#video/common/format.ts";
import type { ActionLog, AddressMode } from "#video/generated/spec.ts";

const SCHEME = /^[a-z]+:\/\//i;

function pageAt(log: ActionLog, timeMs: number): string | undefined {
  const addresses = log.addresses ?? [];
  const shown = addresses.findLast((a) => a.t <= timeMs) ?? addresses[0];
  return shown?.url;
}

const firstPage = (log: ActionLog): string | undefined =>
  log.actions.find((a) => a.kind === "navigate")?.label;

function pathOf(url: string): string {
  if (!URL.canParse(url)) return "";
  const { protocol, pathname, hash } = new URL(url);
  return protocol === "http:" || protocol === "https:" ? `${pathname}${hash}` : "";
}

const hostOf = (typed: string): string => typed.replace(SCHEME, "").split(/[/?#]/, 1)[0];

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
