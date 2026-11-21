import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { chromium } from "playwright-core";
import { z } from "zod";
import { downloadTo, fetchOk } from "./download";
import type { ShellPlatform } from "./engine-downloads";
import { devCache } from "./tool-config";
import { runTool } from "./tools";
import { unzipTo } from "./unzip";

const STAGED_ENGINE = devCache("stage-engine");

const CREDITS_CACHE = devCache("chromium-credits");

const CREDITS_PAGE = "chrome://credits";

const CREDITS_WAIT_MS = 60_000;

const KNOWN_GOOD =
  "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";

const MAC_CHROME = "Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing";

const CHROME_BINARIES: Record<ShellPlatform, string> = {
  linux64: "chrome-linux64/chrome",
  "linux-arm64": "chrome-linux-arm64/chrome",
  "mac-x64": `chrome-mac-x64/${MAC_CHROME}`,
  "mac-arm64": `chrome-mac-arm64/${MAC_CHROME}`,
  win64: "chrome-win64/chrome.exe",
};

const knownGoodSchema = z.object({
  versions: z.array(
    z.object({
      version: z.string(),
      downloads: z.object({
        chrome: z.array(z.object({ platform: z.string(), url: z.string() })).default([]),
      }),
    }),
  ),
});

const installedSchema = z.object({ browser: z.string() });

export type InstalledEngine = z.infer<typeof installedSchema>;

export function installEngine(dir = STAGED_ENGINE): InstalledEngine {
  const lines = runTool(["install-engine", dir]).trim().split("\n");
  return installedSchema.parse(JSON.parse(lines.at(-1) ?? "{}"));
}

export function browserVersion(text: string): string | undefined {
  const line = text.split("\n").find((l) => l.includes("Chrom"));
  return line?.split(/\s+/).find((word) => /^\d+(\.\d+){3}$/.test(word));
}

function shellPlatform(platform = process.platform, arch = process.arch): ShellPlatform {
  const arm = arch === "arm64";
  if (platform === "linux") return arm ? "linux-arm64" : "linux64";
  if (platform === "darwin") return arm ? "mac-arm64" : "mac-x64";
  if (platform === "win32" && !arm) return "win64";
  throw new Error(`Unsupported platform ${platform}/${arch}`);
}

async function chromeUrl(version: string, platform: ShellPlatform): Promise<string> {
  const known = knownGoodSchema.parse(await (await fetchOk(KNOWN_GOOD)).json());
  const url = known.versions
    .find((v) => v.version === version)
    ?.downloads.chrome.find((d) => d.platform === platform)?.url;
  if (!url) throw new Error(`There's no Chrome for Testing ${version} for ${platform}.`);
  return url;
}

async function fullChrome(version: string, platform: ShellPlatform): Promise<string> {
  const dir = path.join(CREDITS_CACHE, version, platform);
  const chrome = path.join(dir, CHROME_BINARIES[platform]);
  if (existsSync(chrome)) return chrome;
  const url = await chromeUrl(version, platform);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  const zip = path.join(dir, "chrome.zip");
  await downloadTo(url, zip);
  await unzipTo(zip, dir);
  rmSync(zip);
  return chrome;
}

export async function writeChromiumCredits(browser: string, output: string): Promise<void> {
  const version = browserVersion(execFileSync(browser, ["--version"], { encoding: "utf8" }));
  if (!version) throw new Error(`${browser} isn't a recording browser.`);
  const launched = await chromium.launch({
    executablePath: await fullChrome(version, shellPlatform()),
  });
  try {
    const page = await launched.newPage();
    await page.goto(CREDITS_PAGE);
    await page.waitForSelector(".product", { state: "attached", timeout: CREDITS_WAIT_MS });
    const html = await page.content();
    mkdirSync(path.dirname(output), { recursive: true });
    writeFileSync(output, html);
    console.log(
      `Saved the licences of Chromium ${version} (${Buffer.byteLength(html)} bytes) to ${output}.`,
    );
  } finally {
    await launched.close();
  }
}
