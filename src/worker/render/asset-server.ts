import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { fileReply, resolveInside } from "../../engine/http-file";

export type AssetServer = { url: string; close: () => Promise<void> };

export async function startAssetServer(root: string): Promise<AssetServer> {
  const server: Server = createServer(async (req, res) => {
    try {
      const file = resolveInside(root, new URL(req.url ?? "/", "http://x").pathname);
      if (!file) {
        res.writeHead(403).end();
        return;
      }
      const reply = await fileReply(file, req.headers.range);
      if (reply.status === 404) {
        res.writeHead(404).end();
        return;
      }
      res.writeHead(reply.status, reply.headers);
      if (req.method === "HEAD" || reply.status === 416) res.end();
      else reply.open().pipe(res);
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
