import { createServer } from "node:net";

export function freeTcpPort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.unref();
    server.on("error", reject);
    server.listen(0, () => {
      const address = server.address();
      if (address === null || typeof address === "string") {
        server.close(() => reject(new Error("Couldn't find a free port.")));
        return;
      }
      server.close(() => resolve(address.port));
    });
  });
}
