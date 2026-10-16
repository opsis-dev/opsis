export const encodePath = (p: string): string => p.split("/").map(encodeURIComponent).join("/");

export const projectRelPath = (ref: string): string =>
  ref.replace(/^project:/, "").replace(/^\/+/, "");

export const baseName = (ref: string): string => ref.replace(/\/+$/, "").split("/").pop() || ref;
