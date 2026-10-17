import React, { createContext, useContext } from "react";
import { staticFile } from "remotion";
import {
  PROJECT_FILE_BASE,
  appRelPath,
  encodePath,
  isProjectRef,
  projectRelPath,
} from "../shared/asset-ref";
import type { ReframeMode } from "../shared/schema/timeline";

type AssetBase = { projectBase?: string };

const AssetContext = createContext<AssetBase>({});

export const AssetProvider: React.FC<{ value: AssetBase; children: React.ReactNode }> = ({
  value,
  children,
}) => <AssetContext.Provider value={value}>{children}</AssetContext.Provider>;

export function resolveAssetRef(ref: string, base: AssetBase): string {
  if (isProjectRef(ref)) {
    const rel = projectRelPath(ref);
    if (base.projectBase === "static:") return staticFile(`project/${rel}`);
    const root = base.projectBase ?? PROJECT_FILE_BASE;
    return root.endsWith("/") ? root + encodePath(rel) : `${root}/${encodePath(rel)}`;
  }
  const app = appRelPath(ref);
  return app === null ? ref : staticFile(app);
}

const isAssetRef = (v: unknown): v is string => typeof v === "string" && /^(project|app):/.test(v);

export function resolveProps<T extends Record<string, unknown>>(props: T, base: AssetBase): T {
  const visit = (v: unknown, depth: number): unknown => {
    if (isAssetRef(v)) return resolveAssetRef(v, base);
    if (depth > 3 || v === null || typeof v !== "object") return v;
    if (Array.isArray(v)) return v.map((x) => visit(x, depth + 1));
    if ("tracks" in (v as object) && depth > 0) return v;
    return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, visit(x, depth + 1)]));
  };
  return visit(props, 0) as T;
}

export const useAssetBase = () => useContext(AssetContext);

export const useAsset = (ref: string | null | undefined): string | null => {
  const base = useAssetBase();
  return ref ? resolveAssetRef(ref, base) : null;
};

const ReframeContext = createContext<ReframeMode>("follow");
export const ReframeProvider = ReframeContext.Provider;
export const useReframe = () => useContext(ReframeContext);

type PreviewQuality = "fast" | "exact";
const QualityContext = createContext<PreviewQuality>("exact");
export const QualityProvider = QualityContext.Provider;
export const useQuality = () => useContext(QualityContext);
