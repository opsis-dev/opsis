import { createContext, useContext } from "react";
import { staticFile } from "remotion";
import {
  PROJECT_FILE_BASE,
  STATIC_PROJECT_BASE,
  encodePath,
  isAssetRef,
  refTarget,
} from "../shared/asset-ref";
import type { PreviewQuality, ReframeMode } from "../shared/schema/timeline";

type AssetBase = { projectBase?: string };

const AssetContext = createContext<AssetBase>({});

export const AssetProvider = AssetContext.Provider;

export function resolveAssetRef(ref: string, base: AssetBase): string {
  const target = refTarget(ref);
  if (target.kind === "app") return staticFile(target.rel);
  if (target.kind !== "project") return ref;
  if (base.projectBase === STATIC_PROJECT_BASE) return staticFile(`project/${target.rel}`);
  const root = base.projectBase ?? PROJECT_FILE_BASE;
  return root.endsWith("/") ? root + encodePath(target.rel) : `${root}/${encodePath(target.rel)}`;
}

const MAX_PROP_DEPTH = 3;

const isNestedTimeline = (v: object, depth: number) => depth > 0 && "tracks" in v;

export function resolveProps<T extends Record<string, unknown>>(props: T, base: AssetBase): T {
  const visit = (v: unknown, depth: number): unknown => {
    if (isAssetRef(v)) return resolveAssetRef(v, base);
    if (depth > MAX_PROP_DEPTH || v === null || typeof v !== "object") return v;
    if (Array.isArray(v)) return v.map((x) => visit(x, depth + 1));
    if (isNestedTimeline(v, depth)) return v;
    return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, visit(x, depth + 1)]));
  };
  return visit(props, 0) as T;
}

export const useAssetBase = () => useContext(AssetContext);

export const useAsset = (ref: string | null | undefined): string | null => {
  const base = useAssetBase();
  return ref ? resolveAssetRef(ref, base) : null;
};

export const DEFAULT_REFRAME: ReframeMode = "follow";
const ReframeContext = createContext<ReframeMode>(DEFAULT_REFRAME);
export const ReframeProvider = ReframeContext.Provider;
export const useReframe = () => useContext(ReframeContext);

export const DEFAULT_QUALITY: PreviewQuality = "exact";
const QualityContext = createContext<PreviewQuality>(DEFAULT_QUALITY);
export const QualityProvider = QualityContext.Provider;
export const useQuality = () => useContext(QualityContext);
