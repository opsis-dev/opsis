import { existsSync } from "node:fs";
import { copyFile, mkdir, readFile } from "node:fs/promises";
import path from "node:path";
import type { BrowserWindow } from "electron";
import { isInside, toPosixRel } from "../engine/project-paths";
import { firstFree } from "../shared/lists";
import { AUDIO_EXTENSIONS, IMAGE_EXTENSIONS, VIDEO_MIME } from "../shared/mime";
import { openDialog } from "./dialogs";
import { glbJson, MODEL_EXTENSIONS, modelFiles } from "./model-files";

const MEDIA_EXTENSIONS = [
  ...Object.keys(VIDEO_MIME),
  ...IMAGE_EXTENSIONS,
  ...AUDIO_EXTENSIONS,
  "lottie",
  "json",
  ...MODEL_EXTENSIONS,
];

const extOf = (file: string) => path.extname(file).slice(1).toLowerCase();

async function modelJson(file: string): Promise<unknown> {
  const bytes = await readFile(file);
  return extOf(file) === "glb" ? glbJson(bytes) : JSON.parse(bytes.toString("utf8"));
}

async function importModel(dir: string, file: string): Promise<string> {
  const { files, refused } = modelFiles(await modelJson(file));
  if (refused.length)
    throw new Error(
      `This model points to files outside its folder or on the internet: ${refused.join(", ")}`,
    );
  const source = path.dirname(file);
  const missing = files.filter((f) => !existsSync(path.join(source, f)));
  if (missing.length)
    throw new Error(`This model needs these files next to it: ${missing.join(", ")}`);
  const models = path.join(dir, "assets", "models");
  await mkdir(models, { recursive: true });
  const folder = uniqueAssetName(models, path.basename(file, path.extname(file)));
  const target = path.join(models, folder);
  for (const f of [path.basename(file), ...files]) {
    const to = path.join(target, f);
    await mkdir(path.dirname(to), { recursive: true });
    await copyFile(path.join(source, f), to);
  }
  return `assets/models/${folder}/${path.basename(file)}`;
}

function uniqueAssetName(assets: string, file: string): string {
  const ext = path.extname(file);
  const stem = path.basename(file, ext);
  return firstFree(
    (n) => (n === 0 ? path.basename(file) : `${stem}-${n}${ext}`),
    (name) => existsSync(path.join(assets, name)),
    0,
  );
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
    if (MODEL_EXTENSIONS.includes(extOf(file))) {
      out.push(await importModel(dir, file));
      continue;
    }
    const name = uniqueAssetName(assets, file);
    await copyFile(file, path.join(assets, name));
    out.push(`assets/${name}`);
  }
  return out;
}
