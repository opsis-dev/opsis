import { spawn } from "node:child_process";
import { once } from "node:events";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { setTimeout as sleep } from "node:timers/promises";
import { APP_BINARY, USER_DATA } from "./support/paths";

const DRIVER_PORT = "4455";
const DRIVER = `http://127.0.0.1:${DRIVER_PORT}`;
const STARTUP_MS = 60_000;
const CLOSE_MS = 15_000;
const POLL_MS = 250;
const W3C_ELEMENT = "element-6066-11e4-a52e-4f735466cecf";

type Reply = { value: Record<string, unknown> & { sessionId?: string } };

async function call(method: string, route: string, body?: unknown): Promise<Reply> {
  const res = await fetch(`${DRIVER}${route}`, {
    method,
    headers: { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  return (await res.json()) as Reply;
}

async function until<T>(what: string, ms: number, check: () => Promise<T | null>): Promise<T> {
  const deadline = Date.now() + ms;
  while (Date.now() < deadline) {
    const found = await check().catch(() => null);
    if (found !== null) return found;
    await sleep(POLL_MS);
  }
  throw new Error(`Timed out waiting for ${what}.`);
}

const element = (session: string, xpath: string) =>
  until(xpath, CLOSE_MS, async () => {
    const reply = await call("POST", `/session/${session}/element`, {
      using: "xpath",
      value: xpath,
    });
    const id = reply.value[W3C_ELEMENT];
    return typeof id === "string" ? id : null;
  });

const click = (session: string, id: string) =>
  call("POST", `/session/${session}/element/${id}/click`, {});

const offScreen =
  process.env.GDK_BACKEND === "x11" &&
  !process.env.WAYLAND_DISPLAY &&
  (process.env.XAUTHORITY ?? "").includes("xvfb-run");
if (!offScreen) throw new Error("Run it with `bun run e2e`: it must run inside xvfb-run.");

const app = spawn(APP_BINARY, [], {
  env: { ...process.env, OPSIS_USER_DATA: USER_DATA, TAURI_WEBDRIVER_PORT: DRIVER_PORT },
  stdio: "inherit",
});
const exited = once(app, "exit");

try {
  const session = await until("the embedded WebDriver", STARTUP_MS, async () => {
    const reply = await call("POST", "/session", { capabilities: {} });
    return reply.value.sessionId ?? null;
  });
  await click(session, await element(session, "//button[@aria-label='Close']"));
  await element(session, "//*[normalize-space()='Are you sure you want to exit?']");
  await click(session, await element(session, `//label[normalize-space()="Don't ask again"]`));
  const exit = await element(session, "//button[normalize-space()='Exit']");
  await click(session, exit).catch(() => "the app quit before the driver could answer");
  const [code] = (await Promise.race([exited, sleep(CLOSE_MS).then(() => ["still running"])])) as [
    number | string,
  ];
  if (code !== 0) throw new Error(`The app didn't close after Exit (${code}).`);
  const db = new DatabaseSync(path.join(USER_DATA, "opsis.db"));
  const row = db.prepare("SELECT value FROM settings WHERE section = 'app'").get() as
    | { value: string }
    | undefined;
  db.close();
  if (!row || (JSON.parse(row.value) as { confirmExit?: boolean }).confirmExit !== false)
    throw new Error("Don't ask again wasn't saved.");
  console.log("✓ closing asks first, remembers Don't ask again and closes");
} finally {
  if (app.exitCode === null) app.kill("SIGKILL");
}
