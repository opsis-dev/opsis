import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync } from "node:fs";
import path from "node:path";
import { chromium } from "playwright-core";

const ROOT = path.resolve(import.meta.dirname, "..");
const SVG = path.join(ROOT, "brand", "logo.svg");
const OUT = path.join(ROOT, "brand", "icons");
export const ICON_SIZES = [16, 24, 32, 48, 64, 128, 256, 512, 1024];

function findShell(dir: string): string | null {
  if (!existsSync(dir)) return null;
  for (const entry of readdirSync(dir)) {
    const p = path.join(dir, entry);
    if (statSync(p).isDirectory()) {
      const hit = findShell(p);
      if (hit) return hit;
    } else if (/^chrome-headless-shell(\.exe)?$/.test(entry)) return p;
  }
  return null;
}

const shell =
  process.env.DEMOFLOW_HEADLESS_SHELL ?? findShell(path.join(ROOT, "resources", "engine"));
if (!shell)
  throw new Error(
    "No recording browser found. Run `bun run make` once (it stages resources/engine) or set DEMOFLOW_HEADLESS_SHELL.",
  );

const svg = readFileSync(SVG, "utf8");
rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });
const browser = await chromium.launch({ executablePath: shell });
const page = await browser.newPage();
for (const size of ICON_SIZES) {
  await page.setViewportSize({ width: size, height: size });
  await page.setContent(
    `<html><body style="margin:0;background:transparent">${svg.replace(
      "<svg ",
      `<svg width="${size}" height="${size}" `,
    )}</body></html>`,
  );
  await page.screenshot({
    path: path.join(OUT, `${size}.png`),
    omitBackground: true,
    clip: { x: 0, y: 0, width: size, height: size },
  });
}
await browser.close();
console.log(`${ICON_SIZES.length} icons in ${path.relative(ROOT, OUT)}`);
