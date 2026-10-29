export const fileName = (path: string): string => path.replace(/\/+$/, "").split("/").pop() || path;
