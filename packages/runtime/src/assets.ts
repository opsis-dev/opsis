export function publicFile(path: string): string {
  const encoded = path.replace(/^\/+/, "").split("/").map(encodeURIComponent).join("/");
  if (typeof document === "undefined") return `/${encoded}`;
  return new URL(encoded, document.baseURI).href;
}
