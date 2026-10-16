import { createHash } from "node:crypto";

export const sha256Hex = (data: string | Uint8Array, length = 64): string =>
  createHash("sha256").update(data).digest("hex").slice(0, length);
