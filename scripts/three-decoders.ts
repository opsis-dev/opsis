import { copyFileSync, mkdirSync, readdirSync, rmSync } from "node:fs";
import path from "node:path";
import { ROOT, VIDEO_PACKAGE_DIR } from "./lib/tool-config";

const FROM = path.join(
  VIDEO_PACKAGE_DIR,
  "node_modules",
  "three",
  "examples",
  "jsm",
  "libs",
  "draco",
  "gltf",
);
const TO = path.join(ROOT, "public", "three", "draco");

rmSync(TO, { recursive: true, force: true });
mkdirSync(TO, { recursive: true });
const files = readdirSync(FROM).filter((f) => /\.(js|wasm)$/.test(f));
for (const file of files) copyFileSync(path.join(FROM, file), path.join(TO, file));
console.log(`copied ${files.length} decoder files to public/three/draco`);
