import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import path from "node:path";

export const MIME: Record<string, string> = {
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
  ".lottie": "application/zip",
};

export type AssetServer = { url: string; close: () => Promise<void> };

export async function startAssetServer(root: string): Promise<AssetServer> {
  const base = path.resolve(root);
  const server: Server = createServer(async (req, res) => {
    try {
      const rel = decodeURIComponent(new URL(req.url ?? "/", "http://x").pathname).replace(
        /^\/+/,
        "",
      );
      const file = path.resolve(base, rel);
      if (!file.startsWith(base + path.sep)) {
        res.writeHead(403).end();
        return;
      }
      const info = await stat(file);
      const type = MIME[path.extname(file).toLowerCase()] ?? "application/octet-stream";
      const headers = {
        "Content-Type": type,
        "Accept-Ranges": "bytes",
        "Access-Control-Allow-Origin": "*",
      };
      const range = /bytes=(\d*)-(\d*)/.exec(req.headers.range ?? "");
      if (range) {
        const start = range[1] ? Number(range[1]) : Math.max(0, info.size - Number(range[2]));
        const end =
          range[1] && range[2] ? Math.min(Number(range[2]), info.size - 1) : info.size - 1;
        res.writeHead(206, {
          ...headers,
          "Content-Range": `bytes ${start}-${end}/${info.size}`,
          "Content-Length": end - start + 1,
        });
        createReadStream(file, { start, end }).pipe(res);
        return;
      }
      res.writeHead(200, { ...headers, "Content-Length": info.size });
      if (req.method === "HEAD") res.end();
      else createReadStream(file).pipe(res);
    } catch {
      res.writeHead(404).end();
    }
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address() as AddressInfo;
  return {
    url: `http://127.0.0.1:${port}/`,
    close: () => new Promise((resolve) => server.close(() => resolve())),
  };
}
