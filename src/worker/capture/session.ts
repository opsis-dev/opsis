import type { Browser, BrowserContext } from "playwright-core";
import type { z } from "zod";
import type { sessionNode } from "../../shared/nodes/browser";
import { captureScaleInitScript, scaledViewport } from "./capture-scale";

export type SessionParams = z.infer<typeof sessionNode.params>;

export type OpenSession = {
  browser: Browser;
  context: BrowserContext;
  physicalViewport: { width: number; height: number };
  logicalViewport: { width: number; height: number };
  close: () => Promise<void>;
};

export async function openSession(
  params: SessionParams,
  opts: { storageState?: string | null; executablePath: string },
): Promise<OpenSession> {
  const { chromium, devices } = await import("playwright-core");
  const logical = { width: params.viewportWidth, height: params.viewportHeight };
  const scale = params.captureScale > 1 ? params.captureScale : 1;
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
  if (scale > 1) await context.addInitScript({ content: captureScaleInitScript(scale, logical) });
  return {
    browser,
    context,
    physicalViewport: physical,
    logicalViewport: logical,
    close: async () => {
      await context.close().catch(() => undefined);
      await browser.close().catch(() => undefined);
    },
  };
}

export const COOKIE_BANNER_SELECTORS = [
  "#onetrust-banner-sdk",
  "#onetrust-consent-sdk",
  "#CybotCookiebotDialog",
  ".cc-window",
  ".cookie-banner",
  "[id*='cookie-banner' i]",
  "[class*='cookie-consent' i]",
  "[aria-label*='cookie' i][role='dialog']",
];

export const CHAT_WIDGET_SELECTORS = [
  "#intercom-container",
  ".intercom-lightweight-app",
  "#hubspot-messages-iframe-container",
  ".drift-frame-controller",
  "#crisp-chatbox",
  "iframe[title*='chat' i]",
  "[id*='chat-widget' i]",
];
