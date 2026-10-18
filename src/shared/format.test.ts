import { describe, expect, it } from "vitest";
import {
  displayAddress,
  formatBytes,
  formatDuration,
  formatPreciseDuration,
  humanize,
  percent,
  percentText,
  singleQuoted,
} from "./format";

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

  it("keeps milliseconds in precise durations", () => {
    expect(formatPreciseDuration(12)).toBe("0.012 s");
    expect(formatPreciseDuration(1500)).toBe("1.5 s");
    expect(formatPreciseDuration(61_234)).toBe("1 min 1.234 s");
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

describe("singleQuoted", () => {
  it("quotes text for a shell, closing and reopening the quotes around each single quote", () => {
    expect(singleQuoted("/home/me/api.sock")).toBe("'/home/me/api.sock'");
    expect(singleQuoted("/home/o'brien/api.sock")).toBe("'/home/o'\\''brien/api.sock'");
  });
});

describe("percent", () => {
  it("writes a fraction as a CSS percentage", () => {
    expect(percent(0.25)).toBe("25%");
    expect(percent(-0.5)).toBe("-50%");
  });

  it("leaves out floating point noise", () => {
    expect(percent(0.07)).toBe("7%");
    expect(percent(0.29)).toBe("29%");
    expect(percent(1 / 3)).toBe("33.333333%");
  });
});

describe("percentText", () => {
  it("rounds a fraction to a whole percentage, capped when asked", () => {
    expect(percentText(0.504)).toBe("50%");
    expect(percentText(1.7)).toBe("170%");
    expect(percentText(1.7, { max: 100 })).toBe("100%");
  });
});
