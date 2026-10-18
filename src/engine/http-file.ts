import { createReadStream, type ReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import path from "node:path";
import { withoutLeadingSlashes } from "../shared/asset-ref";
import { AUDIO_MIME, IMAGE_MIME, VIDEO_MIME } from "../shared/mime";
import { nullIfMissing } from "./json";
import { isInside } from "./project-paths";

const MIME: Partial<Record<string, string>> = {
  ...Object.fromEntries(
    Object.entries({ ...VIDEO_MIME, ...AUDIO_MIME, ...IMAGE_MIME }).map(([ext, type]) => [
      `.${ext}`,
      type,
    ]),
  ),
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

const mimeType = (file: string): string =>
  MIME[path.extname(file).toLowerCase()] ?? "application/octet-stream";

const decodePath = (urlPath: string): string | null => {
  try {
    return decodeURIComponent(urlPath);
  } catch {
    return null;
  }
};

export function resolveInside(root: string, urlPath: string): string | null {
  const decoded = decodePath(urlPath);
  if (decoded === null) return null;
  const base = path.resolve(root);
  const file = path.resolve(base, withoutLeadingSlashes(decoded));
  return isInside(base, file) ? file : null;
}

type ByteRange = { start: number; end: number };

export function parseRange(
  header: string | null | undefined,
  size: number,
): ByteRange | "unsatisfiable" | null {
  const groups = /bytes=(?<first>\d*)-(?<last>\d*)/.exec(header ?? "")?.groups;
  if (!groups) return null;
  const { first, last } = groups;
  if (!first && !last) return "unsatisfiable";
  const start = first ? Number(first) : Math.max(0, size - Number(last));
  const end = first && last ? Math.min(Number(last), size - 1) : size - 1;
  return start > end ? "unsatisfiable" : { start, end };
}

type FileReply =
  | { status: 404 }
  | { status: 416; headers: Record<string, string> }
  | { status: 200 | 206; headers: Record<string, string>; open: () => ReadStream };

export async function fileReply(file: string, rangeHeader?: string | null): Promise<FileReply> {
  const info = await nullIfMissing(stat(file));
  if (!info?.isFile()) return { status: 404 };
  const headers = {
    "Content-Type": mimeType(file),
    "Accept-Ranges": "bytes",
    "Access-Control-Allow-Origin": "*",
  };
  const range = parseRange(rangeHeader, info.size);
  if (range === "unsatisfiable")
    return { status: 416, headers: { ...headers, "Content-Range": `bytes */${info.size}` } };
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
