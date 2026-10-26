import { createHash } from "node:crypto";
import { createReadStream, existsSync, mkdirSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { downloadTo } from "./lib/engine/download";
import { FFMPEG_RELEASE, FFMPEG_RELEASE_URL } from "./lib/engine/paths";
import { ROOT } from "./lib/tool-config";

const CACHE = path.join(os.homedir(), ".cache", "opsis-engine-downloads");
const OUT = path.join(ROOT, "crates", "engine", "src", "downloads.generated.json");

type ChromeUrl = (o: { platform: string; version: null; chromeMode: "headless-shell" }) => string;

function remotionDownloads(): { testedVersion: string; url: ChromeUrl } {
  const require = createRequire(path.join(ROOT, "apps", "sidecar", "package.json"));
  const dir = path.dirname(require.resolve("@remotion/renderer/package.json"));
  const mod = require(path.join(dir, "dist", "browser", "get-chrome-download-url.js")) as {
    TESTED_VERSION: string;
    getChromeDownloadUrl: ChromeUrl;
  };
  return { testedVersion: mod.TESTED_VERSION, url: mod.getChromeDownloadUrl };
}

const GOOGLE = (version: string, platform: string) =>
  `https://storage.googleapis.com/chrome-for-testing-public/${version}/${platform}/chrome-headless-shell-${platform}.zip`;

const REMOTION = (version: string, name: string) =>
  `https://remotion.media/chromium-headless-shell-${name}-${version}.zip?clear`;

const FFMPEG_TARGETS = ["linux-x64", "linux-arm64", "darwin-x64", "darwin-arm64", "win32-x64"];

async function sha256(file: string): Promise<string> {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(file)) hash.update(chunk as Buffer);
  return hash.digest("hex");
}

async function pinned(url: string) {
  const name = createHash("sha256").update(url).digest("hex").slice(0, 16);
  const file = path.join(CACHE, `${name}-${path.basename(new URL(url).pathname)}`);
  if (!existsSync(file)) {
    console.log(`downloading ${url}`);
    await downloadTo(url, file);
  }
  const { size } = await import("node:fs/promises").then((fs) => fs.stat(file));
  return { url, sha256: await sha256(file), size };
}

async function main(): Promise<void> {
  mkdirSync(CACHE, { recursive: true });
  const { testedVersion, url } = remotionDownloads();
  const expected = REMOTION(testedVersion, "mac-arm64");
  if (url({ platform: "mac-arm64", version: null, chromeMode: "headless-shell" }) !== expected)
    throw new Error("Remotion's download addresses changed; update scripts/engine-downloads.ts");
  const shellSources: Array<[string, string, string]> = [
    ["linux64", "remotion", REMOTION(testedVersion, "linux-x64")],
    ["linux64", "google", GOOGLE(testedVersion, "linux64")],
    ["linux-arm64", "remotion", REMOTION(testedVersion, "linux-arm64")],
    ["mac-arm64", "remotion", expected],
    ["mac-x64", "google", GOOGLE(testedVersion, "mac-x64")],
    ["win64", "google", GOOGLE(testedVersion, "win64")],
  ];
  const shells = [];
  for (const [platform, source, address] of shellSources)
    shells.push({ platform, source, ...(await pinned(address)) });
  const ffmpeg = [];
  for (const target of FFMPEG_TARGETS)
    for (const [file, address] of [
      ["ffmpeg", `${FFMPEG_RELEASE_URL}/ffmpeg-${target}.gz`],
      ["ffprobe", `${FFMPEG_RELEASE_URL}/ffprobe-${target}.gz`],
      ["license", `${FFMPEG_RELEASE_URL}/${target}.LICENSE`],
    ])
      ffmpeg.push({ target, file, ...(await pinned(address)) });
  const out = { testedVersion, ffmpegRelease: FFMPEG_RELEASE, shells, ffmpeg };
  writeFileSync(OUT, `${JSON.stringify(out, null, 2)}\n`);
  console.log(`wrote ${path.relative(ROOT, OUT)}`);
}

await main();
