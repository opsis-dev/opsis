import { createServer, type Server } from "node:http";
import { fileReply, resolveInside } from "../../engine/http-file";
import { errorMessage } from "../../shared/errors";

export type AssetServer = { url: string; close: () => Promise<void> };

const logFailure = (asked: string, error: unknown) =>
  process.stderr.write(`[assets] ${asked}: ${errorMessage(error)}\n`);

export async function startAssetServer(root: string): Promise<AssetServer> {
  const server: Server = createServer(async (req, res) => {
    const asked = req.url ?? "/";
    try {
      const file = resolveInside(root, new URL(asked, "http://x").pathname);
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
      else
        reply
          .open()
          .on("error", (error) => {
            logFailure(file, error);
            res.destroy();
          })
          .pipe(res);
    } catch (error) {
      logFailure(asked, error);
      res.writeHead(500).end();
    }
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (!address || typeof address === "string") {
    server.close();
    throw new Error("The asset server didn't get a port.");
  }
  return {
    url: `http://127.0.0.1:${address.port}/`,
    close: () => new Promise((resolve) => server.close(() => resolve())),
  };
}
