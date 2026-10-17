import { Readable } from "node:stream";
import { protocol } from "electron";
import { fileReply, resolveInside } from "../engine/http-file";
import { FILE_SCHEME } from "../shared/asset-ref";

export const APP_SCHEME = "demoflow-app";

const FILE_SOURCE = `${FILE_SCHEME}:`;

const CSP = [
  "default-src 'self'",
  "script-src 'self'",
  "style-src 'self' 'unsafe-inline'",
  `font-src 'self' data: ${FILE_SOURCE}`,
  `img-src 'self' data: blob: ${FILE_SOURCE}`,
  `media-src 'self' blob: ${FILE_SOURCE}`,
  `connect-src 'self' ${FILE_SOURCE} blob: data:`,
  "worker-src 'self' blob:",
].join("; ");

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
  const reply = await fileReply(file, request.headers.get("range"));
  if (reply.status === 404) return new Response("Not found", { status: 404 });
  const body =
    request.method === "HEAD" || reply.status === 416
      ? null
      : (Readable.toWeb(reply.open()) as ReadableStream);
  return new Response(body, {
    status: reply.status,
    headers: { ...reply.headers, ...extraHeaders },
  });
}

export function registerProtocolHandlers(opts: {
  getProjectDir: () => string | null;
  getFontsDir: () => string;
  rendererDir: string;
}): void {
  protocol.handle(FILE_SCHEME, (request) => {
    const url = new URL(request.url);
    const root =
      url.hostname === "project"
        ? opts.getProjectDir()
        : url.hostname === "fonts"
          ? opts.getFontsDir()
          : null;
    if (!root) return new Response("No project open", { status: 404 });
    const file = resolveInside(root, url.pathname);
    if (!file) return new Response("Forbidden", { status: 403 });
    return serveFile(file, request);
  });

  protocol.handle(APP_SCHEME, (request) => {
    const { pathname } = new URL(request.url);
    const file = resolveInside(opts.rendererDir, pathname === "/" ? "index.html" : pathname);
    if (!file) return new Response("Forbidden", { status: 403 });
    return serveFile(
      file,
      request,
      file.endsWith(".html") ? { "Content-Security-Policy": CSP } : {},
    );
  });
}
