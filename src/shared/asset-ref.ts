export const encodePath = (p: string): string => p.split("/").map(encodeURIComponent).join("/");

export const projectRelPath = (ref: string): string =>
  ref.replace(/^project:/, "").replace(/^\/+/, "");

export const baseName = (ref: string): string => ref.replace(/\/+$/, "").split("/").pop() || ref;

const REF_SCHEMES = /^(project:|app:|https?:|data:|blob:|demoflow-file:)/;

export const projectRef = (path: string): string =>
  !path || REF_SCHEMES.test(path) ? path : `project:${path.replace(/^\/+/, "")}`;
