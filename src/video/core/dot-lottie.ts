import type { LottieAnimationData } from "@remotion/lottie";
import { strFromU8, unzipSync, type Unzipped } from "fflate";
import { extensionOf } from "../../shared/asset-ref";
import { isRecord } from "../../shared/guards";
import { IMAGE_MIME } from "../../shared/mime";

const toBase64 = (bytes: Uint8Array) => {
  let out = "";
  for (let i = 0; i < bytes.length; i += 0x8000)
    out += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(out);
};

const parseJson = (bytes: Uint8Array): unknown => JSON.parse(strFromU8(bytes));

const REQUIRED_NUMBERS = ["fr", "w", "h", "op"];

const isAnimation = (value: unknown): value is LottieAnimationData =>
  isRecord(value) && REQUIRED_NUMBERS.every((key) => typeof value[key] === "number");

function asAnimation(value: unknown): LottieAnimationData {
  if (!isAnimation(value)) throw new Error("This file isn't an animation.");
  return value;
}

const isZip = (bytes: Uint8Array) => bytes[0] === 0x50 && bytes[1] === 0x4b;

export const readLottie = (bytes: Uint8Array): LottieAnimationData =>
  isZip(bytes) ? readDotLottie(bytes) : asAnimation(parseJson(bytes));

function firstAnimationId(manifest: unknown): string | undefined {
  const animations = isRecord(manifest) ? manifest.animations : undefined;
  const first = Array.isArray(animations) ? animations[0] : undefined;
  return isRecord(first) && typeof first.id === "string" ? first.id : undefined;
}

function embedImages(data: Record<string, unknown>, files: Unzipped): void {
  if (!Array.isArray(data.assets)) return;
  for (const asset of data.assets) {
    if (!isRecord(asset) || typeof asset.p !== "string" || !asset.p || asset.e === 1) continue;
    const file = files[`images/${asset.p}`] ?? files[`i/${asset.p}`];
    if (!file) continue;
    asset.p = `data:${IMAGE_MIME[extensionOf(asset.p)] ?? "image/png"};base64,${toBase64(file)}`;
    asset.u = "";
    asset.e = 1;
  }
}

export function readDotLottie(bytes: Uint8Array): LottieAnimationData {
  const files = unzipSync(bytes);
  const id = files["manifest.json"]
    ? firstAnimationId(parseJson(files["manifest.json"]))
    : undefined;
  const entry =
    (id && (files[`animations/${id}.json`] ?? files[`a/${id}.json`])) ??
    Object.entries(files).find(([name]) => /^(animations|a)\/[^/]+\.json$/.test(name))?.[1];
  if (!entry) throw new Error("This .lottie file has no animation inside.");
  const data = asAnimation(parseJson(entry));
  embedImages(data, files);
  return data;
}
