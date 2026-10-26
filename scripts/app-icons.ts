import { execFileSync } from "node:child_process";
import { readdirSync, rmSync } from "node:fs";
import path from "node:path";
import { ROOT } from "./lib/tool-config";

const ICONS = path.join(ROOT, "crates", "desktop", "icons");
const DESKTOP = new Set([
  "32x32.png",
  "64x64.png",
  "128x128.png",
  "128x128@2x.png",
  "icon.png",
  "icon.icns",
  "icon.ico",
]);

execFileSync("tauri", ["icon", path.join(ROOT, "brand", "icons", "1024.png"), "-o", ICONS], {
  cwd: path.join(ROOT, "crates", "desktop"),
  stdio: "inherit",
});
for (const name of readdirSync(ICONS))
  if (!DESKTOP.has(name)) rmSync(path.join(ICONS, name), { recursive: true, force: true });
console.log(`kept ${[...DESKTOP].join(", ")}`);
