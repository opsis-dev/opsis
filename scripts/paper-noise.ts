import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { readPaths } from "./lib/setup/store";
import { ROOT } from "./lib/tool-config";

const SOURCE = path.join(
  readPaths().npm["@paper-design/shaders"] ?? "",
  "dist",
  "get-shader-noise-texture.js",
);
const OUT = path.join(ROOT, "crates", "effects", "textures", "paper-noise.png");
const DATA_URL = /"data:image\/png;base64,([A-Za-z0-9+/=]+)"/;

function main(): void {
  const found = DATA_URL.exec(readFileSync(SOURCE, "utf8"));
  if (!found?.[1]) throw new Error(`${SOURCE} has no noise picture`);
  mkdirSync(path.dirname(OUT), { recursive: true });
  writeFileSync(OUT, Buffer.from(found[1], "base64"));
  console.log(`paper noise texture written to ${path.relative(ROOT, OUT)}`);
}

main();
