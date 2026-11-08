import { chmodSync } from "node:fs";
import { type Pinned, verifiedDownload } from "./pins";
import { devCache } from "./tool-config";

export const APPIMAGE_TOOLS_CACHE = devCache("appimage-tools");

type ToolPins = { appimagetool: Pinned; runtime: Pinned };

const APPIMAGETOOL = "https://github.com/AppImage/appimagetool/releases/download/1.9.1";
const RUNTIME = "https://github.com/AppImage/type2-runtime/releases/download/20251108";

const PINS: Record<string, ToolPins> = {
  x86_64: {
    appimagetool: {
      url: `${APPIMAGETOOL}/appimagetool-x86_64.AppImage`,
      sha256: "ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0",
      size: 15092216,
    },
    runtime: {
      url: `${RUNTIME}/runtime-x86_64`,
      sha256: "2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d",
      size: 944632,
    },
  },
  aarch64: {
    appimagetool: {
      url: `${APPIMAGETOOL}/appimagetool-aarch64.AppImage`,
      sha256: "f0837e7448a0c1e4e650a93bb3e85802546e60654ef287576f46c71c126a9158",
      size: 14678536,
    },
    runtime: {
      url: `${RUNTIME}/runtime-aarch64`,
      sha256: "00cbdfcf917cc6c0ff6d3347d59e0ca1f7f45a6df1a428a0d6d8a78664d87444",
      size: 936456,
    },
  },
};

export type AppImageTools = { appimagetool: string; runtime: string };

export async function appImageTools(target: string): Promise<AppImageTools> {
  const arch = target.split("-")[0] ?? "";
  const pins = PINS[arch];
  if (!pins) throw new Error(`No AppImage tools are pinned for ${target}.`);
  const appimagetool = await verifiedDownload(pins.appimagetool, APPIMAGE_TOOLS_CACHE);
  const runtime = await verifiedDownload(pins.runtime, APPIMAGE_TOOLS_CACHE);
  chmodSync(appimagetool, 0o755);
  return { appimagetool, runtime };
}
