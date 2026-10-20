import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { applyPatch, parsePatch, reversePatch, type StructuredPatch } from "diff";
import { isInside } from "../../src/engine/project-paths";
import { ROOT, VIDEO_DIR } from "./tool-config";

export const PATCH_DIR = path.join(ROOT, "scripts", "registry-patches");

type Patched = { text: string; problem?: string };

export function patchSource(source: string, patch: StructuredPatch, where: string): Patched {
  const applied = applyPatch(source, patch);
  if (applied !== false) return { text: applied };
  if (applyPatch(source, reversePatch(patch)) !== false) return { text: source };
  return { text: source, problem: `${where} no longer fits; the element changed upstream` };
}

export function hasReason(patchText: string): boolean {
  const first = patchText.split("\n").find((line) => line.trim());
  return !!first && !/^(diff --git|---|\+\+\+|@@|Index:)/.test(first);
}

const targetOf = (patch: StructuredPatch): string | null => {
  const name = patch.newFileName?.replace(/^b\//, "");
  return name ? path.join(ROOT, name) : null;
};

function applyPatchFile(name: string): string[] {
  const text = readFileSync(path.join(PATCH_DIR, name), "utf8");
  if (!hasReason(text)) return [`${name} must start with a line saying why the patch is needed`];
  const problems: string[] = [];
  for (const patch of parsePatch(text)) {
    const file = targetOf(patch);
    if (!file || !isInside(VIDEO_DIR, file) || !existsSync(file)) {
      problems.push(
        `${name} patches ${patch.newFileName ?? "nothing"}, which isn't an installed element file`,
      );
      continue;
    }
    const before = readFileSync(file, "utf8");
    const { text: after, problem } = patchSource(
      before,
      patch,
      `${name} (${path.relative(ROOT, file)})`,
    );
    if (problem) problems.push(problem);
    else if (after !== before) writeFileSync(file, after);
  }
  return problems;
}

export function applyPatches(): string[] {
  if (!existsSync(PATCH_DIR)) return [];
  return readdirSync(PATCH_DIR)
    .filter((f) => f.endsWith(".patch"))
    .toSorted()
    .flatMap(applyPatchFile);
}
