import { isString } from "./guards";

export const FILE_SCHEME = "opsis-file";

const PROJECT_SCHEME = "project:";

const APP_SCHEME = "app:";

const LIBRARY_SCHEME = "library:";

const EXTERNAL_SCHEMES = ["http:", "https:", "data:", "blob:", `${FILE_SCHEME}:`];

const startsWithAny = (ref: string, schemes: readonly string[]): boolean =>
  schemes.some((scheme) => ref.startsWith(scheme));

export const PROJECT_FILE_BASE = `${FILE_SCHEME}://project/`;

export const STATIC_PROJECT_BASE = "static:";

const encodePath = (p: string): string => p.split("/").map(encodeURIComponent).join("/");

export const joinUrl = (base: string, rel: string): string =>
  `${base.endsWith("/") ? base : `${base}/`}${encodePath(rel)}`;

export const withoutLeadingSlashes = (path: string): string => path.replace(/^\/+/, "");

export const isProjectRef = (ref: string): boolean => ref.startsWith(PROJECT_SCHEME);

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

export const libraryRef = (rel: string): string => `${LIBRARY_SCHEME}${rel}`;

export const libraryRelPath = (ref: string): string | null =>
  ref.startsWith(LIBRARY_SCHEME) ? ref.slice(LIBRARY_SCHEME.length) : null;

export function extensionOf(file: string): string {
  const name = file.slice(file.lastIndexOf("/") + 1);
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

export const baseName = (ref: string): string => ref.replace(/\/+$/, "").split("/").pop() || ref;

export const projectRef = (path: string): string =>
  !path || isProjectRef(path) || refTarget(path).kind !== "project"
    ? path
    : `${PROJECT_SCHEME}${path}`;
