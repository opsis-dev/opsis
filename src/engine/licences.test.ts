import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { completeEngineLicences, shippedCreditsCandidates } from "./licences";
import { CHROMIUM_CREDITS_FILE, enginePaths } from "./paths";
import { tempDirs } from "./test-temp-dirs";

const tempDir = tempDirs("demoflow-licences-");

async function fakeEngine(dir: string): Promise<void> {
  const p = enginePaths(dir);
  for (const file of [p.headlessShells[0], p.ffmpeg, p.ffprobe]) {
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, "");
  }
}

describe("engine licences", () => {
  it("copies the credits that ship with the app into a set-up engine", async () => {
    const root = await tempDir();
    const engine = path.join(root, "user", "engine");
    await fakeEngine(engine);
    const shipped = shippedCreditsCandidates([path.join(root, "resources")]);
    await mkdir(path.dirname(shipped[0]), { recursive: true });
    await writeFile(shipped[0], "<html>credits</html>");
    expect(await completeEngineLicences(engine, shipped)).toBe("copied");
    expect(await readFile(path.join(engine, CHROMIUM_CREDITS_FILE), "utf8")).toBe(
      "<html>credits</html>",
    );
    expect(await completeEngineLicences(engine, shipped)).toBe("present");
  });

  it("does nothing without an engine or without shipped credits", async () => {
    const root = await tempDir();
    const engine = path.join(root, "engine");
    expect(await completeEngineLicences(engine, [])).toBe("no-engine");
    await fakeEngine(engine);
    expect(await completeEngineLicences(engine, shippedCreditsCandidates([root]))).toBe("missing");
  });
});
