import type { BrowserContext } from "playwright-core";
import type { z } from "zod";
import type { sessionNode } from "../../shared/nodes/browser";
import { fitPageToScale, scaledViewport } from "./capture-scale";

type SessionParams = z.infer<typeof sessionNode.params>;

export type OpenSession = {
  context: BrowserContext;
  physicalViewport: { width: number; height: number };
  close: () => Promise<void>;
};

export async function openSession(
  params: SessionParams,
  opts: { storageState?: string | null; executablePath: string },
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
  const context = await browser.newContext({
    viewport: physical,
    deviceScaleFactor: params.dpr,
    locale: params.locale || undefined,
    timezoneId: params.timezone || undefined,
    colorScheme: params.colorScheme,
    userAgent: params.userAgent || (params.mobile ? devices["iPad Pro 11"].userAgent : undefined),
    ...(params.mobile ? { isMobile: true, hasTouch: true } : {}),
    ...(opts.storageState ? { storageState: JSON.parse(opts.storageState) } : {}),
  });
  if (scale > 1) await context.addInitScript(fitPageToScale, { scale, ...logical });
  return {
    context,
    physicalViewport: physical,
    close: async () => {
      await context.close().catch(() => undefined);
      await browser.close().catch(() => undefined);
    },
  };
}
