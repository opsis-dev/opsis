export function fileName(path: string): string {
  return path.replace(/\/+$/, "").split("/").pop() || path;
}
