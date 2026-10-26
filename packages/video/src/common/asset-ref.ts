import { isString } from "#video/common/guards.ts";

const PROJECT_SCHEME = "project:";

const APP_SCHEME = "app:";

const LIBRARY_SCHEME = "library:";

const EXTERNAL_SCHEMES = ["http:", "https:", "data:", "blob:"];

const startsWithAny = (ref: string, schemes: readonly string[]): boolean =>
  schemes.some((scheme) => ref.startsWith(scheme));

export const STATIC_PROJECT_BASE = "static:";

const encodePath = (p: string): string => p.split("/").map(encodeURIComponent).join("/");

export const joinUrl = (base: string, rel: string): string =>
  `${base.endsWith("/") ? base : `${base}/`}${encodePath(rel)}`;

const withoutLeadingSlashes = (path: string): string => path.replace(/^\/+/, "");

const isProjectRef = (ref: string): boolean => ref.startsWith(PROJECT_SCHEME);

export const isAssetRef = (value: unknown): value is string =>
  isString(value) && startsWithAny(value, [PROJECT_SCHEME, APP_SCHEME]);

export const isExternalRef = (ref: string): boolean => startsWithAny(ref, EXTERNAL_SCHEMES);

const ABSOLUTE_PATH = /^(\/|[A-Za-z]:[\\/]|\\\\)/;

type RefTarget =
  | { kind: "project"; rel: string }
  | { kind: "app"; rel: string }
  | { kind: "external"; url: string }
  | { kind: "absolute"; path: string };

export function refTarget(ref: string): RefTarget {
  if (isProjectRef(ref))
    return { kind: "project", rel: withoutLeadingSlashes(ref.slice(PROJECT_SCHEME.length)) };
  if (ref.startsWith(APP_SCHEME)) return { kind: "app", rel: ref.slice(APP_SCHEME.length) };
  if (isExternalRef(ref)) return { kind: "external", url: ref };
  if (ABSOLUTE_PATH.test(ref)) return { kind: "absolute", path: ref };
  return { kind: "project", rel: ref };
}

export function projectRelPath(ref: string): string | null {
  const target = refTarget(ref);
  return target.kind === "project" ? target.rel : null;
}

export function fileVersionKey(ref: string): string | null {
  const rel = projectRelPath(ref.trim());
  return rel === null
    ? null
    : rel
        .split(/[\\/]/)
        .filter((part) => part && part !== ".")
        .join("/");
}

export type FileVersions = Record<string, string>;

export function versionedUrl(url: string, ref: string, versions?: FileVersions): string {
  const key = versions && fileVersionKey(ref);
  const version = key ? versions[key] : undefined;
  return version ? `${url}?v=${encodeURIComponent(version)}` : url;
}

export const libraryRelPath = (ref: string): string | null =>
  ref.startsWith(LIBRARY_SCHEME) ? ref.slice(LIBRARY_SCHEME.length) : null;

export function extensionOf(file: string): string {
  const name = file.slice(file.lastIndexOf("/") + 1);
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

export const baseName = (ref: string): string => ref.replace(/\/+$/, "").split("/").pop() || ref;
