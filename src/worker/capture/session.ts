import type { BrowserContext } from "playwright-core";
import type { NodeParams } from "../../shared/nodes/define";
import type { sessionNode } from "../../shared/nodes/browser";
import { parseJsonWith } from "../../shared/guards";
import { storageStateSchema, type StorageState } from "../../shared/schema/storage-state";
import { fitPageToScale, scaledViewport } from "./capture-scale";

type SessionParams = NodeParams<typeof sessionNode>;

function parseStorageState(text: string): StorageState {
  const parsed = parseJsonWith(storageStateSchema, text);
  if (!parsed?.success)
    throw new Error("The saved sign-in can't be read. Run its Sign in once node again.");
  return parsed.data;
}

export type OpenSession = {
  context: BrowserContext;
  physicalViewport: { width: number; height: number };
  close: () => Promise<void>;
};

export async function openSession(
  params: SessionParams,
  opts: { storageState?: string | null; executablePath: string },
): Promise<OpenSession> {
  const storageState = opts.storageState ? parseStorageState(opts.storageState) : undefined;
  const { chromium, devices } = await import("playwright-core");
  const logical = { width: params.viewportWidth, height: params.viewportHeight };
  const scale = params.captureScale;
  const physical = scaledViewport(logical, scale);
  const browser = await chromium.launch({
    executablePath: opts.executablePath,
    headless: true,
    args: ["--hide-scrollbars", "--disable-features=Translate"],
  });
  const context = await browser
    .newContext({
      viewport: physical,
      deviceScaleFactor: params.dpr,
      locale: params.locale || undefined,
      timezoneId: params.timezone || undefined,
      colorScheme: params.colorScheme,
      userAgent: params.userAgent || (params.mobile ? devices["iPad Pro 11"].userAgent : undefined),
      ...(params.mobile ? { isMobile: true, hasTouch: true } : {}),
      ...(storageState ? { storageState } : {}),
    })
    .catch(async (error: unknown) => {
      await browser.close().catch(() => undefined);
      throw error;
    });
  const close = async () => {
    await context.close().catch(() => undefined);
    await browser.close().catch(() => undefined);
  };
  if (scale > 1)
    await context
      .addInitScript(fitPageToScale, { scale, ...logical })
      .catch(async (error: unknown) => {
        await close();
        throw error;
      });
  return {
    context,
    physicalViewport: physical,
    close,
  };
}
