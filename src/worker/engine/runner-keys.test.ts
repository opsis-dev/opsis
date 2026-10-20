import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { secretFile } from "../../engine/secret-paths";
import { tempDirs } from "../../engine/test-temp-dirs";
import { findById } from "../../shared/lists";
import { GraphBuilder } from "../../shared/templates/build";
import { must } from "../../shared/test-fixtures";
import { hashNodes } from "./runner";

describe("cache keys of skipped nodes", () => {
  it("change everything after a node when Skip is turned on or off", () => {
    const g = new GraphBuilder();
    g.node("bg", "fx.core.solid", 0, 0);
    g.node("spot", "camera.crop", 1, 0);
    g.node("out", "output.render", 2, 0);
    g.link("bg", "spot").link("spot", "out");
    const wf = g.build({ id: "x", name: "x" });
    const keyOf = () => hashNodes(wf, wf.nodes, {}, "/nowhere", "test").get("out");
    const shown = keyOf();
    must(findById(wf.nodes, "spot"), "spot").disabled = true;
    const skipped = keyOf();
    must(findById(wf.nodes, "spot"), "spot").disabled = false;
    expect(skipped).not.toBe(shown);
    expect(keyOf()).toBe(shown);
  });
});

describe("cache keys after a Sign in", () => {
  const project = tempDirs("opsis-sign-in-keys-");

  it("change the take's key when a new sign-in is saved", async () => {
    const dir = await project();
    const g = new GraphBuilder();
    g.node("open", "browser.open", 0, 0);
    g.node("login", "browser.signIn", 1, 0, { secretName: "admin" });
    g.node("go", "browser.navigate", 2, 0, { url: "https://example.com" });
    g.node("rec", "capture.record", 3, 0);
    g.chain(["open", "login", "go", "rec"]);
    const wf = g.build({ id: "x", name: "x" });
    const keyOf = () => hashNodes(wf, wf.nodes, {}, dir, "test").get("rec");
    const unsaved = keyOf();
    const file = secretFile(dir, "admin");
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, "first");
    const saved = keyOf();
    await writeFile(file, "a later sign-in");
    expect(saved).not.toBe(unsaved);
    expect(keyOf()).not.toBe(saved);
  });
});
