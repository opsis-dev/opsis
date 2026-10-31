import { publicFile } from "@opsis/runtime/assets.ts";
import { createContext, useContext } from "react";
import { isString } from "#video/common/guards.ts";
import type { ElementProps } from "#video/common/spec.ts";
import type { ReframeMode } from "#video/generated/spec.ts";

const APP_SCHEME = "app:";

export function assetUrl(address: string): string {
  return address.startsWith(APP_SCHEME) ? publicFile(address.slice(APP_SCHEME.length)) : address;
}

const MAX_PROP_DEPTH = 3;

function isNestedTimeline(v: object, depth: number) {
  return depth > 0 && "tracks" in v;
}

export function withAppFiles(props: ElementProps): ElementProps {
  function visit(v: unknown, depth: number): unknown {
    if (isString(v)) return assetUrl(v);
    if (depth > MAX_PROP_DEPTH || v === null || typeof v !== "object") return v;
    if (Array.isArray(v)) return v.map((x) => visit(x, depth + 1));
    if (isNestedTimeline(v, depth)) return v;
    return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, visit(x, depth + 1)]));
  }
  return Object.fromEntries(Object.entries(props).map(([k, v]) => [k, visit(v, 1)]));
}

export const DEFAULT_REFRAME: ReframeMode = "follow";
const ReframeContext = createContext<ReframeMode>(DEFAULT_REFRAME);
export const ReframeProvider = ReframeContext.Provider;
export function useReframe(): ReframeMode {
  return useContext(ReframeContext);
}
