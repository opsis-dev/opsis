import path from "node:path";
import { $, browser, expect } from "@wdio/globals";
import { api } from "../support/api";
import { PROJECTS } from "../support/paths";

const WAIT_MS = 30_000;

function clock() {
  return $("//*[@data-timeline-root]//button[contains(., ' / ')]");
}

function hasActiveNode() {
  return browser.execute(() =>
    [...document.querySelectorAll(".react-flow__node *")].some((el) =>
      el.classList.contains("ring-primary/70"),
    ),
  );
}

function hasHighlightedBar() {
  return browser.execute(
    () => document.querySelector("[data-timeline-root] .brightness-125") !== null,
  );
}

describe("timeline", function () {
  this.timeout(120_000);

  it("plays on its own clock and marks the nodes and bars under the playhead", async () => {
    const dir = path.join(PROJECTS, "changelog");
    expect((await api("POST", "/open", { dir })).status).toBe(200);
    await $("[data-timeline-root]").waitForExist({ timeout: WAIT_MS });
    await browser.waitUntil(hasActiveNode, {
      timeout: WAIT_MS,
      timeoutMsg: "no node was marked at the playhead",
    });
    expect(await hasHighlightedBar()).toBe(true);
    const before = await clock().getText();
    await $('[data-timeline-root] button[aria-label="Play"]').click();
    await browser.waitUntil(async () => (await clock().getText()) !== before, {
      timeout: WAIT_MS,
      timeoutMsg: "the playhead didn't move",
    });
    expect((await browser.getWindowHandles()).length).toBe(1);
    await $('[data-timeline-root] button[aria-label="Pause"]').click();
    await expect($('[data-timeline-root] button[aria-label="Play"]')).toExist();
  });
});
