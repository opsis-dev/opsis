import React, { createContext, useContext } from "react";
import { staticFile } from "remotion";

export type AssetBase = { projectBase?: string };

const AssetContext = createContext<AssetBase>({});

export const AssetProvider: React.FC<{ value: AssetBase; children: React.ReactNode }> = ({
  value,
  children,
}) => <AssetContext.Provider value={value}>{children}</AssetContext.Provider>;

const encodePath = (p: string) => p.split("/").map(encodeURIComponent).join("/");

export function resolveAssetRef(ref: string, base: AssetBase): string {
  if (ref.startsWith("project:")) {
    const rel = ref.slice("project:".length).replace(/^\/+/, "");
    if (base.projectBase === "static:") return staticFile(`project/${rel}`);
    const root = base.projectBase ?? "demoflow-file://project/";
    return root.endsWith("/") ? root + encodePath(rel) : `${root}/${encodePath(rel)}`;
  }
  if (ref.startsWith("app:")) return staticFile(ref.slice("app:".length));
  return ref;
}

export const isAssetRef = (v: unknown): v is string =>
  typeof v === "string" && /^(project|app):/.test(v);

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

export type ReframeMode = "follow" | "center" | "fit";
const ReframeContext = createContext<ReframeMode>("follow");
export const ReframeProvider = ReframeContext.Provider;
export const useReframe = () => useContext(ReframeContext);

export type PreviewQuality = "fast" | "exact";
const QualityContext = createContext<PreviewQuality>("exact");
export const QualityProvider = QualityContext.Provider;
export const useQuality = () => useContext(QualityContext);
