import type { LottieAnimationData } from "@remotion/lottie";
import { strFromU8, unzipSync } from "fflate";

type LottieAsset = { id?: string; p?: string; u?: string; e?: number };
type DotLottieManifest = { animations?: Array<{ id: string }> };

const MIME: Record<string, string> = {
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  webp: "image/webp",
  gif: "image/gif",
  svg: "image/svg+xml",
};

const toBase64 = (bytes: Uint8Array) => {
  let out = "";
  for (let i = 0; i < bytes.length; i += 0x8000)
    out += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(out);
};

export function readDotLottie(bytes: Uint8Array): LottieAnimationData {
  const files = unzipSync(bytes);
  const manifest = files["manifest.json"]
    ? (JSON.parse(strFromU8(files["manifest.json"])) as DotLottieManifest)
    : {};
  const id = manifest.animations?.[0]?.id;
  const entry =
    (id && (files[`animations/${id}.json`] ?? files[`a/${id}.json`])) ??
    Object.entries(files).find(([name]) => /^(animations|a)\/[^/]+\.json$/.test(name))?.[1];
  if (!entry) throw new Error("This .lottie file has no animation inside.");
  const data = JSON.parse(strFromU8(entry)) as LottieAnimationData & { assets?: LottieAsset[] };
  for (const asset of data.assets ?? []) {
    if (!asset.p || asset.e === 1) continue;
    const file = files[`images/${asset.p}`] ?? files[`i/${asset.p}`];
    if (!file) continue;
    const ext = asset.p.split(".").pop()?.toLowerCase() ?? "png";
    asset.p = `data:${MIME[ext] ?? "image/png"};base64,${toBase64(file)}`;
    asset.u = "";
    asset.e = 1;
  }
  return data;
}
