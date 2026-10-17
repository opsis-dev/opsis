import { describe, expect, it } from "vitest";
import { displayAddress, formatBytes, formatDuration, humanize, parseNumberText } from "./format";

describe("formatBytes", () => {
  it("uses binary units with one decimal", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(2048)).toBe("2.0 KB");
    expect(formatBytes(5_242_880)).toBe("5.0 MB");
    expect(formatBytes(3 * 1024 ** 3)).toBe("3.0 GB");
  });

  it("shows nothing for unknown sizes", () => {
    expect(formatBytes(-1)).toBe("");
    expect(formatBytes(Number.NaN)).toBe("");
  });
});

describe("formatDuration", () => {
  it("writes short durations in seconds and minutes", () => {
    expect(formatDuration(400)).toBe("0.4 s");
    expect(formatDuration(1500)).toBe("1.5 s");
    expect(formatDuration(14_520)).toBe("14.5 s");
    expect(formatDuration(65_000)).toBe("1 min 5 s");
    expect(formatDuration(119_600)).toBe("2 min");
  });

  it("keeps milliseconds in the precise style", () => {
    expect(formatDuration(12, "precise")).toBe("0.012 s");
    expect(formatDuration(1500, "precise")).toBe("1.5 s");
    expect(formatDuration(61_234, "precise")).toBe("1 min 1.234 s");
  });
});

describe("humanize", () => {
  it("turns keys into sentence-case labels", () => {
    expect(humanize("fooBar")).toBe("Foo bar");
    expect(humanize("end_time")).toBe("End time");
    expect(humanize("ease-in-out")).toBe("Ease in out");
    expect(humanize("x2Y")).toBe("X2 y");
    expect(humanize("macOS")).toBe("Mac OS");
  });
});

describe("displayAddress", () => {
  it("drops the scheme, www and the trailing slash", () => {
    expect(displayAddress("https://www.saucedemo.com/")).toBe("saucedemo.com");
    expect(displayAddress("https://www.example.com/a/?x=1")).toBe("example.com/a");
    expect(displayAddress("https://www.example.com/a/?x=1", { keepQuery: true })).toBe(
      "example.com/a?x=1",
    );
  });

  it("leaves addresses with variables readable", () => {
    expect(displayAddress("{{baseUrl}}/login/")).toBe("{{baseUrl}}/login");
  });
});

describe("parseNumberText", () => {
  it("reads comma decimals and gives nothing for blank text", () => {
    expect(parseNumberText("1,5")).toBe(1.5);
    expect(parseNumberText(" 2 ")).toBe(2);
    expect(parseNumberText("")).toBeNull();
    expect(parseNumberText("   ")).toBeNull();
    expect(parseNumberText("12 dB")).toBeNull();
  });

  it("ignores units when lenient", () => {
    expect(parseNumberText("-12 dB", { lenient: true })).toBe(-12);
    expect(parseNumberText("5s", { lenient: true })).toBe(5);
    expect(parseNumberText("dB", { lenient: true })).toBeNull();
  });
});
