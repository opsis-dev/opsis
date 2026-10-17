export const FILE_SCHEME = "demoflow-file";

export const PROJECT_FILE_BASE = `${FILE_SCHEME}://project/`;

export const encodePath = (p: string): string => p.split("/").map(encodeURIComponent).join("/");

export const projectRelPath = (ref: string): string =>
  ref.replace(/^project:/, "").replace(/^\/+/, "");

export const isProjectRef = (ref: string): boolean => ref.startsWith("project:");

export const projectFileUrl = (ref: string): string =>
  `${PROJECT_FILE_BASE}${encodePath(projectRelPath(ref))}`;

export const isExternalRef = (ref: string): boolean =>
  /^(https?:|data:|blob:|demoflow-file:)/.test(ref);

export function extensionOf(file: string): string {
  const name = file.slice(file.lastIndexOf("/") + 1);
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

export const baseName = (ref: string): string => ref.replace(/\/+$/, "").split("/").pop() || ref;

const REF_SCHEMES = /^(project:|app:|https?:|data:|blob:|demoflow-file:)/;

export const projectRef = (path: string): string =>
  !path || REF_SCHEMES.test(path) ? path : `project:${path.replace(/^\/+/, "")}`;
