import { createWriteStream } from "node:fs";
import { Readable, type Transform } from "node:stream";
import { pipeline } from "node:stream/promises";
import { writeViaTemp } from "./atomic";

export async function fetchOk(url: string, init?: RequestInit): Promise<Response> {
  const res = await fetch(url, init);
  if (!res.ok) throw new Error(`Request failed (${res.status}): ${url}`);
  return res;
}

export async function downloadTo(
  url: string,
  dest: string,
  opts: {
    onBytes?: (received: number, total: number | null) => void;
    signal?: AbortSignal;
    transform?: Transform;
  } = {},
): Promise<void> {
  const res = await fetchOk(url, { signal: opts.signal });
  if (!res.body) throw new Error(`Download failed, the reply was empty: ${url}`);
  const total = Number(res.headers.get("content-length")) || null;
  let received = 0;
  const body = Readable.from(res.body).on("data", (chunk: Buffer) => {
    received += chunk.length;
    opts.onBytes?.(received, total);
  });
  await writeViaTemp(dest, (temp) =>
    pipeline(body, ...(opts.transform ? [opts.transform] : []), createWriteStream(temp)),
  );
}
