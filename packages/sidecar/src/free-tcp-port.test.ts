import { createServer } from "node:net";
import { describe, expect, it } from "vitest";
import { freeTcpPort } from "./free-tcp-port";

describe("freeTcpPort", () => {
  it("returns a port that can be listened on again", async () => {
    const port = await freeTcpPort();
    expect(port).toBeGreaterThan(1023);
    const server = createServer();
    await new Promise<void>((resolve, reject) => {
      server.on("error", reject);
      server.listen(port, resolve);
    });
    await new Promise<void>((resolve) => server.close(() => resolve()));
  });
});
