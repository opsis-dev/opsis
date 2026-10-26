import http from "node:http";
import type { AddressInfo } from "node:net";
import path from "node:path";
import { $, browser, expect } from "@wdio/globals";
import { api } from "../support/api";
import { PROJECTS } from "../support/paths";

const PAGE_WIDTH = 800;
const PAGE_HEIGHT = 600;
const BUTTON = { x: 200, y: 130 };
const WAIT_MS = 30_000;
const PAGE = `<!doctype html><html><body style="margin:0;background:#fff"><button style="position:absolute;left:100px;top:100px;width:200px;height:60px;font-size:20px">Save draft</button></body></html>`;

type Picking = { __picked?: string };

describe("streamed views", function () {
  this.timeout(120_000);
  const server = http.createServer((_, res) => {
    res.setHeader("content-type", "text/html");
    res.end(PAGE);
  });
  let url = "";

  before(async () => {
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    url = `http://127.0.0.1:${(server.address() as AddressInfo).port}/`;
  });

  after(() => server.close());

  it("streams an open project's preview into the Preview tab", async () => {
    const dir = path.join(PROJECTS, "changelog");
    expect((await api("POST", "/open", { dir })).status).toBe(200);
    await browser.keys(["Control", "1"]);
    await $('canvas[aria-label="Preview"]').waitForExist({ timeout: WAIT_MS });
    await browser.waitUntil(
      () =>
        browser.execute(
          () =>
            (document.querySelector<HTMLCanvasElement>('canvas[aria-label="Preview"]')?.width ??
              0) > 0,
        ),
      { timeout: WAIT_MS, timeoutMsg: "the preview drew no frame" },
    );
  });

  it("picks an element on a page streamed into its own window", async () => {
    const main = await browser.getWindowHandle();
    await browser.execute(
      (address: string, width: number, height: number) => {
        const page = window as unknown as Picking & {
          __TAURI_INTERNALS__: { invoke: (cmd: string, args: unknown) => Promise<unknown> };
        };
        page.__picked = "waiting";
        page.__TAURI_INTERNALS__
          .invoke("pick_element", { request: { url: address, viewport: { width, height } } })
          .then(
            (result) => (page.__picked = JSON.stringify(result)),
            (error: unknown) => (page.__picked = `error ${String(error)}`),
          );
      },
      url,
      PAGE_WIDTH,
      PAGE_HEIGHT,
    );
    const pickedSoFar = () => browser.execute(() => (window as unknown as Picking).__picked);
    await browser.waitUntil(
      async () =>
        (await browser.getWindowHandles()).length > 1 || (await pickedSoFar()) !== "waiting",
      { timeout: WAIT_MS },
    );
    expect(await pickedSoFar()).toBe("waiting");
    const view = (await browser.getWindowHandles()).find((h) => h !== main);
    if (!view) throw new Error("no view window");
    await browser.switchToWindow(view);
    await $("canvas").waitForExist({ timeout: WAIT_MS });
    await browser.waitUntil(
      () => browser.execute(() => (document.querySelector("canvas")?.width ?? 0) > 0),
      { timeout: WAIT_MS },
    );
    const point = await browser.execute(
      (width: number, height: number, x: number, y: number) => {
        const canvas = document.querySelector("canvas");
        if (!canvas) return null;
        const box = canvas.getBoundingClientRect();
        const scale = Math.min(box.width / width, box.height / height);
        return {
          x: Math.round(box.left + (box.width - width * scale) / 2 + x * scale),
          y: Math.round(box.top + (box.height - height * scale) / 2 + y * scale),
        };
      },
      PAGE_WIDTH,
      PAGE_HEIGHT,
      BUTTON.x,
      BUTTON.y,
    );
    if (!point) throw new Error("no canvas");
    await browser.performActions([
      {
        type: "pointer",
        id: "mouse",
        parameters: { pointerType: "mouse" },
        actions: [
          { type: "pointerMove", origin: "viewport", x: point.x, y: point.y, duration: 0 },
          { type: "pause", duration: 600 },
          { type: "pointerDown", button: 0 },
          { type: "pointerUp", button: 0 },
        ],
      },
    ]);
    await browser.switchToWindow(main);
    await browser.waitUntil(
      () => browser.execute(() => (window as unknown as Picking).__picked !== "waiting"),
      { timeout: WAIT_MS },
    );
    const picked = await browser.execute(() => (window as unknown as Picking).__picked);
    expect(picked).toContain("Save draft");
  });
});
