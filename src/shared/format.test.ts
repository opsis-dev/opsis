import { describe, expect, it } from "vitest";
import {
  displayAddress,
  formatBytes,
  formatDuration,
  formatPreciseDuration,
  formatTimeAgo,
  formatWhen,
  humanize,
  pascalCase,
  percent,
  percentText,
  plural,
  signedText,
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

describe("pascalCase", () => {
  it("joins kebab and snake case words", () => {
    expect(pascalCase("my_icon-x")).toBe("MyIconX");
    expect(pascalCase("arrow-up-0-1")).toBe("ArrowUp01");
  });
});

describe("humanize", () => {
  it("turns keys into sentence-case labels", () => {
    expect(humanize("fooBar")).toBe("Foo bar");
    expect(humanize("end_time")).toBe("End time");
    expect(humanize("ease-in-out")).toBe("Ease in out");
    expect(humanize("x2Y")).toBe("X2 y");
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

describe("formatWhen", () => {
  it("writes a date and time the same way on every computer", () => {
    expect(formatWhen("2026-10-01T09:41:00")).toBe("1 Oct 2026, 09:41");
  });

  it("shows text that isn't a date as it is", () => {
    expect(formatWhen("next week")).toBe("next week");
  });
});

describe("formatTimeAgo", () => {
  const now = Date.UTC(2026, 9, 18, 12);

  it("says just now for the last few seconds", () => {
    expect(formatTimeAgo(now - 2000, now)).toBe("just now");
  });

  it("names the largest unit that fits", () => {
    expect(formatTimeAgo(now - 5 * 60_000, now)).toBe("5 minutes ago");
    expect(formatTimeAgo(now - 26 * 3_600_000, now)).toBe("yesterday");
  });
});

describe("plural", () => {
  it("adds an s, or uses the given plural, except for one", () => {
    expect(plural(1, "clip")).toBe("1 clip");
    expect(plural(3, "clip")).toBe("3 clips");
    expect(plural(0, "font family", "font families")).toBe("0 font families");
  });
});

describe("signedText", () => {
  it("writes a typographic minus, a plus only when asked, and no sign on zero", () => {
    expect(signedText(-14.25, { digits: 1 })).toBe("−14.3");
    expect(signedText(1.24, { digits: 1, plus: true })).toBe("+1.2");
    expect(signedText(1.24, { digits: 1 })).toBe("1.2");
    expect(signedText(-0.01, { digits: 1, plus: true })).toBe("0.0");
    expect(signedText(-23)).toBe("−23");
  });
});
