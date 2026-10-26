import { mkdirSync } from "node:fs";
import path from "node:path";
import { browser } from "@wdio/globals";
import { APP_BINARY, SHOTS, USER_DATA } from "./src/support/paths";
import { freshRun } from "./src/support/user-data";

const OFF_SCREEN =
  process.env.GDK_BACKEND === "x11" &&
  !process.env.WAYLAND_DISPLAY &&
  (process.env.XAUTHORITY ?? "").includes("xvfb-run");

if (!OFF_SCREEN)
  throw new Error(
    "Run the e2e tests with `bun run e2e`: they must run inside xvfb-run, off the screen.",
  );

const STARTUP_MS = 120_000;

process.env.OPSIS_USER_DATA = USER_DATA;

export const config: WebdriverIO.Config = {
  runner: "local",
  specs: [
    "./src/specs/start.spec.ts",
    "./src/specs/examples.spec.ts",
    "./src/specs/views.spec.ts",
    "./src/specs/export.spec.ts",
  ],
  maxInstances: 1,
  services: [
    [
      "@wdio/tauri-service",
      {
        driverProvider: "embedded",
        appBinaryPath: APP_BINARY,
        startTimeout: STARTUP_MS,
        captureFrontendLogs: true,
        captureBackendLogs: true,
      },
    ],
  ],
  capabilities: [{ browserName: "tauri" }],
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: { ui: "bdd", timeout: STARTUP_MS },
  autoXvfb: false,
  onPrepare: freshRun,
  afterTest: async (test, _context, { passed }) => {
    if (passed) return;
    mkdirSync(SHOTS, { recursive: true });
    await browser
      .saveScreenshot(path.join(SHOTS, `${test.parent} ${test.title}.png`))
      .catch((error: unknown) => console.warn("No screenshot: the app is gone.", error));
  },
  before: async () => {
    const [main] = await browser.getWindowHandles();
    await browser.switchToWindow(main);
  },
};
