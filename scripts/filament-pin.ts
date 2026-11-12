import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import {
  FILAMENT_PIN_FILE,
  type FilamentArchive,
  type FilamentPins,
  fetchFilament,
} from "./lib/filament";

const REPOSITORY = "google/filament";
const VERSION = "v1.80.1";
const FETCH_FLAG = "--fetch";

const PLATFORMS: Record<string, string> = {
  "linux-x64": "linux",
  "linux-arm64": "arm-linux",
  "darwin-arm64": "mac",
  "darwin-x64": "mac",
  "win32-x64": "windows",
};

type Asset = { name: string; size: number; digest: string; browser_download_url: string };

function releaseAssets(): Asset[] {
  const text = execFileSync(
    "gh",
    ["api", `repos/${REPOSITORY}/releases/tags/${VERSION}`, "-q", ".assets"],
    { encoding: "utf8" },
  );
  return JSON.parse(text) as Asset[];
}

function pin(): FilamentPins {
  const assets = releaseAssets();
  const archives: Record<string, FilamentArchive> = {};
  for (const [platform, suffix] of Object.entries(PLATFORMS)) {
    const asset = assets.find((a) => a.name === `filament-${VERSION}-${suffix}.tgz`);
    if (!asset) throw new Error(`Filament ${VERSION} has no ${suffix} archive`);
    archives[platform] = {
      url: asset.browser_download_url,
      size: asset.size,
      sha256: asset.digest.replace(/^sha256:/, ""),
    };
  }
  const pins = { version: VERSION, archives };
  writeFileSync(FILAMENT_PIN_FILE, `${JSON.stringify(pins, null, 2)}\n`);
  return pins;
}

const pins = pin();
console.log(
  `Filament ${pins.version}: pinned ${Object.keys(pins.archives).length} archives in ${FILAMENT_PIN_FILE}`,
);
if (process.argv.includes(FETCH_FLAG)) {
  console.log(`Filament: unpacked into ${fetchFilament(pins)}`);
}
