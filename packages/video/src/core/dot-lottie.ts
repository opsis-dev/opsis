import { strFromU8, unzipSync, type Unzipped } from "fflate";
import { z } from "zod";
import { isRecord, isString } from "#video/common/guards.ts";
import { extensionOf, IMAGE_MIME } from "#video/common/mime.ts";

function toBase64(bytes: Uint8Array) {
  let out = "";
  for (let i = 0; i < bytes.length; i += 0x8000)
    out += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(out);
}

function parseJson(bytes: Uint8Array): unknown {
  return JSON.parse(strFromU8(bytes));
}

const assetSchema = z.looseObject({ p: z.string().optional(), e: z.number().optional() });

type LottieAsset = z.infer<typeof assetSchema>;

const animationSchema = z.looseObject({
  fr: z.number(),
  ip: z.number().optional(),
  w: z.number(),
  h: z.number(),
  op: z.number(),
  assets: z.array(assetSchema).optional(),
});

export type LottieAnimation = z.infer<typeof animationSchema>;

function asAnimation(value: unknown): LottieAnimation {
  const parsed = animationSchema.safeParse(value);
  if (!parsed.success) throw new Error("This file isn't an animation.");
  return parsed.data;
}

function isZip(bytes: Uint8Array) {
  return bytes[0] === 0x50 && bytes[1] === 0x4b;
}

export function readLottie(bytes: Uint8Array): LottieAnimation {
  return isZip(bytes) ? readDotLottie(bytes) : asAnimation(parseJson(bytes));
}

function firstAnimationId(manifest: unknown): string | undefined {
  const animations = isRecord(manifest) ? manifest.animations : undefined;
  const first = Array.isArray(animations) ? animations[0] : undefined;
  return isRecord(first) && isString(first.id) ? first.id : undefined;
}

function embedImage(asset: LottieAsset, files: Unzipped): LottieAsset {
  if (!asset.p || asset.e === 1) return asset;
  const file = files[`images/${asset.p}`] ?? files[`i/${asset.p}`];
  if (!file) return asset;
  const mime = IMAGE_MIME[extensionOf(asset.p)] ?? "image/png";
  return { ...asset, p: `data:${mime};base64,${toBase64(file)}`, u: "", e: 1 };
}

function withEmbeddedImages(data: LottieAnimation, files: Unzipped): LottieAnimation {
  return data.assets
    ? { ...data, assets: data.assets.map((asset) => embedImage(asset, files)) }
    : data;
}

function readDotLottie(bytes: Uint8Array): LottieAnimation {
  const files = unzipSync(bytes);
  const id = files["manifest.json"]
    ? firstAnimationId(parseJson(files["manifest.json"]))
    : undefined;
  const entry =
    (id && (files[`animations/${id}.json`] ?? files[`a/${id}.json`])) ??
    Object.entries(files).find(([name]) => /^(animations|a)\/[^/]+\.json$/.test(name))?.[1];
  if (!entry) throw new Error("This .lottie file has no animation inside.");
  return withEmbeddedImages(asAnimation(parseJson(entry)), files);
}

export type LottieTiming = {
  frame: number;
  fps: number;
  speed: number;
  loop: boolean;
  backwards: boolean;
};

export function animationFrame(data: LottieAnimation, timing: LottieTiming): number {
  const first = data.ip ?? 0;
  const length = Math.max(1, data.op - first);
  const last = length - 1;
  const played = (timing.frame / timing.fps) * timing.speed * data.fr;
  const wrapped = timing.loop ? ((played % length) + length) % length : Math.max(played, 0);
  const along = Math.min(wrapped, last);
  return first + (timing.backwards ? last - along : along);
}
