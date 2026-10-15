import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import path from "node:path";
import { Readable } from "node:stream";
import { protocol } from "electron";
import { MIME } from "../worker/render/asset-server";

export const FILE_SCHEME = "demoflow-file";

const CSP = [
  "default-src 'self'",
  "script-src 'self'",
  "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
  "font-src 'self' data: https://fonts.gstatic.com demoflow-file:",
  "img-src 'self' data: blob: demoflow-file:",
  "media-src 'self' blob: demoflow-file:",
  "connect-src 'self' demoflow-file: blob: data:",
  "worker-src 'self' blob:",
].join("; ");
export const APP_SCHEME = "demoflow-app";

export function registerSchemes(): void {
  const privileges = {
    standard: true,
    secure: true,
    supportFetchAPI: true,
    corsEnabled: true,
    stream: true,
  };
  protocol.registerSchemesAsPrivileged([
    { scheme: FILE_SCHEME, privileges },
    { scheme: APP_SCHEME, privileges: { ...privileges, allowServiceWorkers: false } },
  ]);
}

async function serveFile(
  file: string,
  request: Request,
  extraHeaders: Record<string, string> = {},
): Promise<Response> {
  let info;
  try {
    info = await stat(file);
    if (!info.isFile()) return new Response("Not found", { status: 404 });
  } catch {
    return new Response("Not found", { status: 404 });
  }
  const type =
    MIME[path.extname(file).toLowerCase()] ??
    (file.endsWith(".html")
      ? "text/html"
      : file.endsWith(".js")
        ? "text/javascript"
        : file.endsWith(".css")
          ? "text/css"
          : "application/octet-stream");
  const headers: Record<string, string> = {
    "Content-Type": type,
    "Accept-Ranges": "bytes",
    "Access-Control-Allow-Origin": "*",
    ...extraHeaders,
  };
  const range = /bytes=(\d*)-(\d*)/.exec(request.headers.get("range") ?? "");
  if (range) {
    const start = range[1] ? Number(range[1]) : Math.max(0, info.size - Number(range[2]));
    const end = range[1] && range[2] ? Math.min(Number(range[2]), info.size - 1) : info.size - 1;
    const body = Readable.toWeb(createReadStream(file, { start, end })) as ReadableStream;
    return new Response(body, {
      status: 206,
      headers: {
        ...headers,
        "Content-Range": `bytes ${start}-${end}/${info.size}`,
        "Content-Length": String(end - start + 1),
      },
    });
  }
  const body = Readable.toWeb(createReadStream(file)) as ReadableStream;
  return new Response(body, {
    status: 200,
    headers: { ...headers, "Content-Length": String(info.size) },
  });
}

const inside = (root: string, file: string) => file === root || file.startsWith(root + path.sep);

export function registerProtocolHandlers(opts: {
  getProjectDir: () => string | null;
  rendererDir: string;
}): void {
  protocol.handle(FILE_SCHEME, (request) => {
    const url = new URL(request.url);
    const root = url.hostname === "project" ? opts.getProjectDir() : null;
    if (!root) return new Response("No project open", { status: 404 });
    const file = path.resolve(root, decodeURIComponent(url.pathname).replace(/^\/+/, ""));
    if (!inside(path.resolve(root), file)) return new Response("Forbidden", { status: 403 });
    return serveFile(file, request);
  });

  protocol.handle(APP_SCHEME, (request) => {
    const url = new URL(request.url);
    const rel = decodeURIComponent(url.pathname).replace(/^\/+/, "") || "index.html";
    const file = path.resolve(opts.rendererDir, rel);
    if (!inside(path.resolve(opts.rendererDir), file))
      return new Response("Forbidden", { status: 403 });
    return serveFile(
      file,
      request,
      file.endsWith(".html") ? { "Content-Security-Policy": CSP } : {},
    );
  });
}
