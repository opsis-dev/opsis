import type { LottieAnimationData } from "@remotion/lottie";
import { strFromU8, unzipSync } from "fflate";
import { extensionOf } from "../../shared/asset-ref";
import { IMAGE_MIME } from "../../shared/mime";

type LottieAsset = { id?: string; p?: string; u?: string; e?: number };
type DotLottieManifest = { animations?: Array<{ id: string }> };

const toBase64 = (bytes: Uint8Array) => {
  let out = "";
  for (let i = 0; i < bytes.length; i += 0x8000)
    out += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(out);
};

const parseJson = <T>(bytes: Uint8Array): T => JSON.parse(strFromU8(bytes));

const isZip = (bytes: Uint8Array) => bytes[0] === 0x50 && bytes[1] === 0x4b;

export const readLottie = (bytes: Uint8Array): LottieAnimationData =>
  isZip(bytes) ? readDotLottie(bytes) : parseJson<LottieAnimationData>(bytes);

export function readDotLottie(bytes: Uint8Array): LottieAnimationData {
  const files = unzipSync(bytes);
  const manifest = files["manifest.json"]
    ? parseJson<DotLottieManifest>(files["manifest.json"])
    : {};
  const id = manifest.animations?.[0]?.id;
  const entry =
    (id && (files[`animations/${id}.json`] ?? files[`a/${id}.json`])) ??
    Object.entries(files).find(([name]) => /^(animations|a)\/[^/]+\.json$/.test(name))?.[1];
  if (!entry) throw new Error("This .lottie file has no animation inside.");
  const data = parseJson<LottieAnimationData & { assets?: LottieAsset[] }>(entry);
  for (const asset of data.assets ?? []) {
    if (!asset.p || asset.e === 1) continue;
    const file = files[`images/${asset.p}`] ?? files[`i/${asset.p}`];
    if (!file) continue;
    asset.p = `data:${IMAGE_MIME[extensionOf(asset.p)] ?? "image/png"};base64,${toBase64(file)}`;
    asset.u = "";
    asset.e = 1;
  }
  return data;
}
