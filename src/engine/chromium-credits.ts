import { execFile } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";
import { chromium } from "playwright-core";
import { z } from "zod";
import { downloadTo, fetchOk } from "./download";
import { remotionPlatform, type RemotionPlatform } from "./paths";

const run = promisify(execFile);

const CREDITS_CACHE = path.join(os.homedir(), ".cache", "demoflow-chromium-credits");

const KNOWN_GOOD =
  "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";

const CHROME_BIN: Partial<Record<RemotionPlatform, string>> = {
  linux64: "chrome-linux64/chrome",
  "mac-x64":
    "chrome-mac-x64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
  "mac-arm64":
    "chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
  win64: "chrome-win64/chrome.exe",
};

const knownGoodSchema = z.object({
  versions: z.array(
    z.object({
      version: z.string(),
      downloads: z.object({
        chrome: z.array(z.object({ platform: z.string(), url: z.string() })).optional(),
      }),
    }),
  ),
});

async function chromiumVersion(browser: string): Promise<string> {
  const { stdout } = await run(browser, ["--version"], { encoding: "utf8" });
  const version = /(\d+\.\d+\.\d+\.\d+)/.exec(stdout)?.[1];
  if (!version) throw new Error(`Couldn't read the browser version from "${stdout.trim()}".`);
  return version;
}

async function findChromeUrl(version: string, platform: RemotionPlatform): Promise<string> {
  const res = await fetchOk(KNOWN_GOOD);
  const index = knownGoodSchema.parse(await res.json());
  const url = index.versions
    .find((v) => v.version === version)
    ?.downloads.chrome?.find((d) => d.platform === platform)?.url;
  if (!url) throw new Error(`Chrome for Testing ${version} has no ${platform} build.`);
  return url;
}

async function installChrome(url: string, dir: string): Promise<void> {
  await rm(dir, { recursive: true, force: true });
  await mkdir(dir, { recursive: true });
  const zip = path.join(dir, "chrome.zip");
  await downloadTo(url, zip);
  if (process.platform === "win32") await run("tar", ["-xf", zip, "-C", dir]);
  else await run("unzip", ["-q", zip, "-d", dir]);
  await rm(zip, { force: true });
}

async function fullChrome(version: string): Promise<string> {
  const platform = remotionPlatform();
  const binary = CHROME_BIN[platform];
  if (!binary)
    throw new Error(`No Chrome for Testing build for ${process.platform}/${process.arch}.`);
  const dir = path.join(CREDITS_CACHE, version, platform);
  const bin = path.join(dir, binary);
  if (!existsSync(bin)) await installChrome(await findChromeUrl(version, platform), dir);
  return bin;
}

export async function writeChromiumCredits(
  browser: string,
  output: string,
): Promise<{ version: string; bytes: number }> {
  const version = await chromiumVersion(browser);
  const chrome = await fullChrome(version);
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
