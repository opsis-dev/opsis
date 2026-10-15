import { execFileSync } from "node:child_process";
import { createWriteStream, existsSync } from "node:fs";
import { mkdir, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { chromium } from "playwright-core";

const KNOWN_GOOD =
  "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";

const CFT_PLATFORM: Record<string, string> = {
  "linux-x64": "linux64",
  "linux-arm64": "linux64",
  "darwin-x64": "mac-x64",
  "darwin-arm64": "mac-arm64",
  "win32-x64": "win64",
  "win32-arm64": "win64",
};

const CHROME_BIN: Record<string, string> = {
  linux64: "chrome-linux64/chrome",
  "mac-x64":
    "chrome-mac-x64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
  "mac-arm64":
    "chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
  win64: "chrome-win64/chrome.exe",
};

type KnownGood = {
  versions: Array<{
    version: string;
    downloads: { chrome?: Array<{ platform: string; url: string }> };
  }>;
};

export function chromiumVersion(browser: string): string {
  const out = execFileSync(browser, ["--version"], { encoding: "utf8" });
  const version = /(\d+\.\d+\.\d+\.\d+)/.exec(out)?.[1];
  if (!version) throw new Error(`Couldn't read the browser version from "${out.trim()}".`);
  return version;
}

async function fullChrome(version: string, cacheDir: string): Promise<string> {
  const platform = CFT_PLATFORM[`${process.platform}-${process.arch}`];
  if (!platform)
    throw new Error(`No Chrome for Testing build for ${process.platform}/${process.arch}.`);
  const dir = path.join(cacheDir, version, platform);
  const bin = path.join(dir, CHROME_BIN[platform]);
  if (existsSync(bin)) return bin;
  const res = await fetch(KNOWN_GOOD);
  if (!res.ok) throw new Error(`Couldn't read the Chrome for Testing index (${res.status}).`);
  const index = (await res.json()) as KnownGood;
  const url = index.versions
    .find((v) => v.version === version)
    ?.downloads.chrome?.find((d) => d.platform === platform)?.url;
  if (!url) throw new Error(`Chrome for Testing ${version} has no ${platform} build.`);
  await rm(dir, { recursive: true, force: true });
  await mkdir(dir, { recursive: true });
  const zip = path.join(dir, "chrome.zip");
  const download = await fetch(url);
  if (!download.ok || !download.body)
    throw new Error(`Download failed (${download.status}): ${url}`);
  await pipeline(Readable.fromWeb(download.body as never), createWriteStream(zip));
  if (process.platform === "win32") execFileSync("tar", ["-xf", zip, "-C", dir]);
  else execFileSync("unzip", ["-q", zip, "-d", dir]);
  await rm(zip, { force: true });
  return bin;
}

export async function writeChromiumCredits(
  browser: string,
  output: string,
  cacheDir: string,
): Promise<{ version: string; bytes: number }> {
  const version = chromiumVersion(browser);
  const chrome = await fullChrome(version, cacheDir);
  const instance = await chromium.launch({ executablePath: chrome, args: ["--no-sandbox"] });
  try {
    const page = await instance.newPage();
    await page.goto("chrome://credits", { timeout: 60_000 });
    await page.waitForSelector(".product", { timeout: 60_000 });
    const html = await page.content();
    await mkdir(path.dirname(output), { recursive: true });
    await writeFile(output, html);
    return { version, bytes: Buffer.byteLength(html) };
  } finally {
    await instance.close();
  }
}
