export const FILE_SCHEME = "demoflow-file";

const PROJECT_SCHEME = "project:";

const APP_SCHEME = "app:";

const EXTERNAL_SCHEMES = ["http:", "https:", "data:", "blob:", `${FILE_SCHEME}:`];

const startsWithAny = (ref: string, schemes: readonly string[]): boolean =>
  schemes.some((scheme) => ref.startsWith(scheme));

export const PROJECT_FILE_BASE = `${FILE_SCHEME}://project/`;

export const encodePath = (p: string): string => p.split("/").map(encodeURIComponent).join("/");

export const withoutLeadingSlashes = (path: string): string => path.replace(/^\/+/, "");

export const isProjectRef = (ref: string): boolean => ref.startsWith(PROJECT_SCHEME);

export const projectRelPath = (ref: string): string =>
  withoutLeadingSlashes(isProjectRef(ref) ? ref.slice(PROJECT_SCHEME.length) : ref);

export const appRelPath = (ref: string): string | null =>
  ref.startsWith(APP_SCHEME) ? ref.slice(APP_SCHEME.length) : null;

export const projectFileUrl = (ref: string): string =>
  `${PROJECT_FILE_BASE}${encodePath(projectRelPath(ref))}`;

export const isExternalRef = (ref: string): boolean => startsWithAny(ref, EXTERNAL_SCHEMES);

export function extensionOf(file: string): string {
  const name = file.slice(file.lastIndexOf("/") + 1);
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

export const baseName = (ref: string): string => ref.replace(/\/+$/, "").split("/").pop() || ref;

const REF_SCHEMES = [PROJECT_SCHEME, APP_SCHEME, ...EXTERNAL_SCHEMES];

export const projectRef = (path: string): string =>
  !path || startsWithAny(path, REF_SCHEMES)
    ? path
    : `${PROJECT_SCHEME}${withoutLeadingSlashes(path)}`;
