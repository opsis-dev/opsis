import { existsSync } from "node:fs";
import { copyFile, mkdir } from "node:fs/promises";
import path from "node:path";
import type { BrowserWindow } from "electron";
import { isInside, toPosixRel } from "../engine/project-paths";
import { openDialog } from "./dialogs";

const MEDIA_EXTENSIONS = [
  "mp4",
  "mov",
  "webm",
  "mkv",
  "png",
  "jpg",
  "jpeg",
  "webp",
  "gif",
  "svg",
  "mp3",
  "wav",
  "m4a",
  "aac",
  "ogg",
  "flac",
  "lottie",
  "json",
];

function uniqueAssetName(assets: string, file: string): string {
  const ext = path.extname(file);
  const stem = path.basename(file, ext);
  let name = path.basename(file);
  for (let n = 1; existsSync(path.join(assets, name)); n++) name = `${stem}-${n}${ext}`;
  return name;
}

export async function importAssets(
  win: BrowserWindow | null,
  dir: string,
  paths?: string[],
): Promise<string[]> {
  const files = paths?.length
    ? paths
    : await openDialog(win, {
        title: "Import media",
        properties: ["openFile", "multiSelections"],
        filters: [
          { name: "Media", extensions: MEDIA_EXTENSIONS },
          { name: "All files", extensions: ["*"] },
        ],
      });
  if (!files.length) return [];
  const assets = path.join(dir, "assets");
  await mkdir(assets, { recursive: true });
  const out: string[] = [];
  for (const file of files) {
    if (isInside(dir, file)) {
      out.push(toPosixRel(dir, file));
      continue;
    }
    const name = uniqueAssetName(assets, file);
    await copyFile(file, path.join(assets, name));
    out.push(`assets/${name}`);
  }
  return out;
}
