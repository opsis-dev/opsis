import { browser } from "@wdio/globals";

const ENGINE_MS = 60_000;

type Invoking = {
  __TAURI_INTERNALS__: { invoke: (cmd: string, args?: unknown) => Promise<unknown> };
};

type Status = { complete: boolean };

export async function engineReady(): Promise<void> {
  await browser.waitUntil(
    () =>
      browser.execute(() =>
        (window as unknown as Invoking).__TAURI_INTERNALS__
          .invoke("get_engine_status")
          .then((status) => (status as Status).complete),
      ),
    { timeout: ENGINE_MS, timeoutMsg: "the seeded engine never passed its checks" },
  );
}
