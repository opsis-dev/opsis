import type { BrowserContext } from "playwright-core";
import { onAbort } from "../../shared/errors";
import type { openBrowserNode } from "../../shared/nodes/browser";
import type { NodeParams } from "../../shared/nodes/define";
import type { Viewport } from "../../shared/schema/actions";
import { fitPageToScale, scaledViewport } from "./capture-scale";

type BrowserParams = NodeParams<typeof openBrowserNode>;

export type OpenSession = {
  context: BrowserContext;
  physicalViewport: Viewport;
  close: () => Promise<void>;
};

export async function openSession(
  params: BrowserParams,
  opts: { executablePath: string; signal?: AbortSignal },
): Promise<OpenSession> {
  const { chromium, devices } = await import("playwright-core");
  const logical = { width: params.viewportWidth, height: params.viewportHeight };
  const scale = params.captureScale;
  const physical = scaledViewport(logical, scale);
  const browser = await chromium.launch({
    executablePath: opts.executablePath,
    headless: true,
    args: ["--hide-scrollbars", "--disable-features=Translate"],
  });
  let context: BrowserContext | undefined;
  const stopOnAbort = onAbort(opts.signal, () => void close());
  async function close() {
    stopOnAbort();
    await context?.close().catch(() => undefined);
    await browser.close().catch(() => undefined);
  }
  try {
    context = await browser.newContext({
      viewport: physical,
      deviceScaleFactor: params.dpr,
      locale: params.locale.trim() || Intl.DateTimeFormat().resolvedOptions().locale,
      timezoneId: params.timezone.trim() || undefined,
      colorScheme: params.colorScheme,
      userAgent: params.userAgent || (params.mobile ? devices["iPad Pro 11"].userAgent : undefined),
      ...(params.mobile ? { isMobile: true, hasTouch: true } : {}),
    });
    if (scale > 1) await context.addInitScript(fitPageToScale, { scale, ...logical });
  } catch (error) {
    await close();
    throw error;
  }
  return { context, physicalViewport: physical, close };
}
