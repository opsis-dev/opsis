import { createHash } from "node:crypto";
import { stableJson } from "../shared/hash";

export const sha256Hex = (data: string | Uint8Array, length = 64): string =>
  createHash("sha256").update(data).digest("hex").slice(0, length);

export const versionedHash = (version: string, value: unknown, length = 24): string =>
  sha256Hex(`${version}\0${stableJson(value)}`, length);
