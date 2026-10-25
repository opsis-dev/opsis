import { describe, expect, it } from "vitest";
import { isLocale, isTimeZone } from "#core/locale";

describe("languages and time zones", () => {
  it("accepts language tags and refuses other text", () => {
    expect(isLocale("de-DE")).toBe(true);
    expect(isLocale("ja")).toBe(true);
    expect(isLocale("xx_INVALID!!")).toBe(false);
    expect(isLocale("en-US, de")).toBe(false);
  });

  it("accepts time zone names and refuses made-up ones", () => {
    expect(isTimeZone("Asia/Tokyo")).toBe(true);
    expect(isTimeZone("UTC")).toBe(true);
    expect(isTimeZone("Mars/Olympus_Mons")).toBe(false);
  });
});
