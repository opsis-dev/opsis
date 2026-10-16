import { createReadStream, type ReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import path from "node:path";

const MIME: Record<string, string> = {
  ".mp4": "video/mp4",
  ".m4v": "video/mp4",
  ".mov": "video/quicktime",
  ".webm": "video/webm",
  ".mp3": "audio/mpeg",
  ".m4a": "audio/mp4",
  ".aac": "audio/aac",
  ".wav": "audio/wav",
  ".ogg": "audio/ogg",
  ".flac": "audio/flac",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".webp": "image/webp",
  ".gif": "image/gif",
  ".svg": "image/svg+xml",
  ".json": "application/json",
  ".woff2": "font/woff2",
  ".woff": "font/woff",
  ".ttf": "font/ttf",
  ".otf": "font/otf",
  ".txt": "text/plain; charset=utf-8",
  ".lottie": "application/zip",
  ".html": "text/html",
  ".js": "text/javascript",
  ".mjs": "text/javascript",
  ".css": "text/css",
};

export const mimeType = (file: string): string =>
  MIME[path.extname(file).toLowerCase()] ?? "application/octet-stream";

export function resolveInside(root: string, urlPath: string): string | null {
  const base = path.resolve(root);
  const file = path.resolve(base, decodeURIComponent(urlPath).replace(/^\/+/, ""));
  return file === base || file.startsWith(base + path.sep) ? file : null;
}

export type ByteRange = { start: number; end: number };

export function parseRange(header: string | null | undefined, size: number): ByteRange | null {
  const m = /bytes=(\d*)-(\d*)/.exec(header ?? "");
  if (!m) return null;
  const start = m[1] ? Number(m[1]) : Math.max(0, size - Number(m[2]));
  const end = m[1] && m[2] ? Math.min(Number(m[2]), size - 1) : size - 1;
  return { start, end };
}

export type FileReply =
  | { status: 404 }
  | { status: 200 | 206; headers: Record<string, string>; open: () => ReadStream };

export async function fileReply(file: string, rangeHeader?: string | null): Promise<FileReply> {
  const info = await stat(file).catch(() => null);
  if (!info?.isFile()) return { status: 404 };
  const headers = {
    "Content-Type": mimeType(file),
    "Accept-Ranges": "bytes",
    "Access-Control-Allow-Origin": "*",
  };
  const range = parseRange(rangeHeader, info.size);
  if (!range)
    return {
      status: 200,
      headers: { ...headers, "Content-Length": String(info.size) },
      open: () => createReadStream(file),
    };
  return {
    status: 206,
    headers: {
      ...headers,
      "Content-Range": `bytes ${range.start}-${range.end}/${info.size}`,
      "Content-Length": String(range.end - range.start + 1),
    },
    open: () => createReadStream(file, range),
  };
}
