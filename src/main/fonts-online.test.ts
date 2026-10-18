import { afterEach, describe, expect, it, vi } from "vitest";
import { downloadOnlineFont, fontCatalog } from "./fonts-online";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("online fonts without a connection", () => {
  it("asks to check the connection when the font list can't be reached", async () => {
    vi.stubGlobal("fetch", () => Promise.reject(new TypeError("fetch failed")));
    await expect(fontCatalog()).rejects.toThrow(
      "Couldn't load the font list. Check your internet connection.",
    );
  });

  it("asks to check the connection when a font can't be reached", async () => {
    vi.stubGlobal("fetch", () => Promise.reject(new TypeError("fetch failed")));
    await expect(downloadOnlineFont("lato")).rejects.toThrow(
      "Couldn't find that font online. Check your internet connection.",
    );
  });
});

describe("online fonts that aren't there", () => {
  it("says the font wasn't found without blaming the connection", async () => {
    vi.stubGlobal("fetch", () => Promise.resolve(new Response("", { status: 404 })));
    await expect(downloadOnlineFont("nope")).rejects.toThrow(/^Couldn't find that font online\.$/);
  });
});
