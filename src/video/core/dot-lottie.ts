import type { LottieAnimationData } from "@remotion/lottie";
import { strFromU8, unzipSync, type Unzipped } from "fflate";
import { z } from "zod";
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

const assetSchema = z.looseObject({ p: z.string().optional(), e: z.number().optional() });

type LottieAsset = z.infer<typeof assetSchema>;

const animationSchema = z.looseObject({
  fr: z.number(),
  w: z.number(),
  h: z.number(),
  op: z.number(),
  assets: z.array(assetSchema).optional(),
});

type Animation = z.infer<typeof animationSchema>;

function asAnimation(value: unknown): Animation {
  const parsed = animationSchema.safeParse(value);
  if (!parsed.success) throw new Error("This file isn't an animation.");
  return parsed.data;
}

const isZip = (bytes: Uint8Array) => bytes[0] === 0x50 && bytes[1] === 0x4b;

export const readLottie = (bytes: Uint8Array): LottieAnimationData =>
  isZip(bytes) ? readDotLottie(bytes) : asAnimation(parseJson(bytes));

function firstAnimationId(manifest: unknown): string | undefined {
  const animations = isRecord(manifest) ? manifest.animations : undefined;
  const first = Array.isArray(animations) ? animations[0] : undefined;
  return isRecord(first) && typeof first.id === "string" ? first.id : undefined;
}

function embedImage(asset: LottieAsset, files: Unzipped): LottieAsset {
  if (!asset.p || asset.e === 1) return asset;
  const file = files[`images/${asset.p}`] ?? files[`i/${asset.p}`];
  if (!file) return asset;
  const mime = IMAGE_MIME[extensionOf(asset.p)] ?? "image/png";
  return { ...asset, p: `data:${mime};base64,${toBase64(file)}`, u: "", e: 1 };
}

const withEmbeddedImages = (data: Animation, files: Unzipped): Animation =>
  data.assets ? { ...data, assets: data.assets.map((asset) => embedImage(asset, files)) } : data;

function readDotLottie(bytes: Uint8Array): LottieAnimationData {
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
