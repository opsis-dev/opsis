import { $, browser, expect } from "@wdio/globals";
import { engineReady } from "../support/engine";

type Invoking = {
  __TAURI_INTERNALS__: { invoke: (cmd: string, args?: unknown) => Promise<unknown> };
};

type Checked = { path: string; version: string; sha256: string } | null;

type Settings = { engine: { browser: Checked; ffmpeg: Checked } };

describe("start screen", function () {
  this.timeout(90_000);

  it("shows the start actions and the templates from Rust", async () => {
    await expect($("//*[normalize-space()='New project…']")).toBeDisplayed();
    await expect($("//*[normalize-space()='Changelog clip']")).toBeDisplayed();
    await expect($("//*[normalize-space()='Sample: to-do app']")).toBeDisplayed();
  });

  it("checks the engine programs it was given and saves each one's hash", async () => {
    await engineReady();
    const settings = (await browser.execute(() =>
      (window as unknown as Invoking).__TAURI_INTERNALS__.invoke("get_settings"),
    )) as Settings;
    for (const tool of [settings.engine.browser, settings.engine.ffmpeg]) {
      expect(tool?.sha256).toMatch(/^[0-9a-f]{64}$/);
      expect(tool?.version).not.toBe("");
    }
  });
});
