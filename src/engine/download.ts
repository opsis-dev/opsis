import { createWriteStream } from "node:fs";
import { rename, rm } from "node:fs/promises";
import { Readable, type Transform } from "node:stream";
import { pipeline } from "node:stream/promises";
import type { ReadableStream } from "node:stream/web";

export async function downloadTo(
  url: string,
  dest: string,
  opts: {
    onBytes?: (received: number, total: number | null) => void;
    signal?: AbortSignal;
    transform?: Transform;
  } = {},
): Promise<void> {
  const res = await fetch(url, { signal: opts.signal });
  if (!res.ok || !res.body) throw new Error(`Download failed (${res.status}): ${url}`);
  const total = Number(res.headers.get("content-length")) || null;
  let received = 0;
  const body = Readable.fromWeb(res.body as ReadableStream<Uint8Array>).on(
    "data",
    (chunk: Buffer) => {
      received += chunk.length;
      opts.onBytes?.(received, total);
    },
  );
  const tmp = `${dest}.part`;
  try {
    await pipeline(body, ...(opts.transform ? [opts.transform] : []), createWriteStream(tmp));
    await rename(tmp, dest);
  } finally {
    await rm(tmp, { force: true });
  }
}
