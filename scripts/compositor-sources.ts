import { createHash } from "node:crypto";
import { createReadStream, existsSync, mkdirSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  appVersion,
  assetName,
  readCompositorSources,
  sourcesNote,
} from "./lib/compositor-sources";
import { downloadTo } from "./lib/download";

const manifest = readCompositorSources();
const out =
  process.argv[2] ??
  path.join(os.homedir(), ".cache", "opsis-compositor-sources", manifest.remotion);

async function sha256(file: string): Promise<string> {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(file)) hash.update(chunk as Buffer);
  return hash.digest("hex");
}

mkdirSync(out, { recursive: true });
const sums: string[] = [];
for (const source of manifest.sources) {
  const file = path.join(out, assetName(source.url));
  if (!existsSync(file)) {
    console.log(`downloading ${source.url}`);
    await downloadTo(source.url, file);
  }
  sums.push(`${await sha256(file)}  ${path.basename(file)}`);
}
writeFileSync(path.join(out, "SHA256SUMS"), `${sums.join("\n")}\n`);
writeFileSync(path.join(out, "SOURCES.md"), sourcesNote(manifest, appVersion()));
console.log(`Saved the compositor's FFmpeg sources in ${out}.`);
