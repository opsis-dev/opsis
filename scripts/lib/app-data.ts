import os from "node:os";
import path from "node:path";
import { z } from "zod";
import { readJsonWith } from "./read-json";
import { ROOT } from "./tool-config";

const appName = readJsonWith(
  z.object({ productName: z.string() }),
  path.join(ROOT, "package.json"),
).productName;

function electronAppData(): string {
  if (process.platform === "darwin")
    return path.join(os.homedir(), "Library", "Application Support");
  if (process.platform === "win32")
    return process.env.APPDATA ?? path.join(os.homedir(), "AppData", "Roaming");
  return process.env.XDG_CONFIG_HOME ?? path.join(os.homedir(), ".config");
}

export const devUserData = (): string => path.join(electronAppData(), appName);
