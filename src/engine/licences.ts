import { existsSync } from "node:fs";
import { copyFile } from "node:fs/promises";
import path from "node:path";
import { CHROMIUM_CREDITS_FILE, ENGINE_LICENCES_FOLDER, engineStatus } from "./paths";

export const shippedCreditsCandidates = (roots: string[]): string[] =>
  roots.flatMap((root) => [
    path.join(root, ENGINE_LICENCES_FOLDER, CHROMIUM_CREDITS_FILE),
    path.join(root, "engine", CHROMIUM_CREDITS_FILE),
  ]);

export async function completeEngineLicences(
  engineDir: string,
  candidates: string[],
): Promise<"present" | "copied" | "missing" | "no-engine"> {
  if (!engineStatus(engineDir).complete) return "no-engine";
  const target = path.join(engineDir, CHROMIUM_CREDITS_FILE);
  if (existsSync(target)) return "present";
  const source = candidates.find((c) => path.resolve(c) !== path.resolve(target) && existsSync(c));
  if (!source) return "missing";
  await copyFile(source, target);
  return "copied";
}
