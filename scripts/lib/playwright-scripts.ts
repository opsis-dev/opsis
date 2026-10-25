import { writeFileSync } from "node:fs";
import path from "node:path";
import { playwrightInternals } from "./playwright-bundle";
import { ROOT } from "./tool-config";

export const PLAYWRIGHT_DIR = path.join(ROOT, "crates", "capture", "src", "playwright");

export function playwrightFiles(): Record<string, string> {
  const pw = playwrightInternals();
  const tables = {
    version: pw.version,
    keyboard: pw.keyboardLayout,
    keypadLocation: pw.keypadLocation,
    macEditingCommands: pw.macEditingCommands,
    fontFamilies: pw.fontFamilies,
    switches: pw.switches,
  };
  return {
    "injected.generated.js": pw.injectedSource,
    "clock.generated.js": pw.clockSource,
    "tables.generated.json": `${JSON.stringify(tables, null, 1)}\n`,
  };
}

export function writePlaywrightFiles(): void {
  for (const [name, text] of Object.entries(playwrightFiles())) {
    writeFileSync(path.join(PLAYWRIGHT_DIR, name), text);
    console.log(`wrote ${path.relative(ROOT, path.join(PLAYWRIGHT_DIR, name))}`);
  }
}
