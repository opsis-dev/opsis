import { existsSync, mkdirSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
  clearStaleWorkerTemps,
  makeWorkerTemp,
  removeWorkerTemp,
  workerTempName,
} from "./worker-temp";

const roots: string[] = [];
const root = () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), "demoflow-worker-temp-test-"));
  roots.push(dir);
  return dir;
};
afterEach(() => {
  for (const dir of roots.splice(0)) rmSync(dir, { recursive: true, force: true });
});

describe("worker temp folders", () => {
  it("names a folder after the app process, the kind of worker and the run", () => {
    expect(workerTempName("run", "c6bad309-c2f3-469a-bbc5-6a493245e67e", 4242)).toBe(
      "4242-run-c6bad309",
    );
  });

  it("makes and removes a worker's folder with everything in it", () => {
    const dir = makeWorkerTemp(root(), workerTempName("stills", "abc"));
    mkdirSync(path.join(dir, "react-motion-render123", "frames"), { recursive: true });
    removeWorkerTemp(dir);
    expect(existsSync(dir)).toBe(false);
  });

  it("clears folders left by app processes that are gone, and keeps live ones", () => {
    const base = root();
    for (const name of [
      `${process.pid}-run-mine0000`,
      "111-run-alive000",
      "222-run-gone0000",
      "stray-folder",
    ])
      mkdirSync(path.join(base, name));
    const removed = clearStaleWorkerTemps(base, (pid) => pid === 111);
    expect(removed.toSorted()).toEqual(["222-run-gone0000", "stray-folder"]);
    expect(readdirSync(base).toSorted()).toEqual(
      ["111-run-alive000", `${process.pid}-run-mine0000`].toSorted(),
    );
  });

  it("does nothing when the folder doesn't exist yet", () => {
    expect(clearStaleWorkerTemps(path.join(root(), "missing"))).toEqual([]);
  });
});
