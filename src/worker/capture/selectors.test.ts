import { describe, expect, it } from "vitest";
import type { Locator, Page } from "playwright-core";
import { autoCandidates } from "./selectors";

type Matchers = { name?: RegExp; hasText?: RegExp };

type Call = { fn: string; arg: unknown; opts?: Matchers };

function stubPage() {
  const calls: Call[] = [];
  const loc = (fn: string, arg: unknown, opts?: Matchers): Locator => {
    calls.push({ fn, arg, opts });
    return {
      filter: (o: Matchers) => loc(`${fn}.filter`, arg, o),
      locator: (a: unknown) => loc(`${fn}.locator`, a),
      last: () => loc(`${fn}.last`, arg),
      first: () => loc(`${fn}.first`, arg),
    } as unknown as Locator;
  };
  const page = {
    getByRole: (r: string, o?: Matchers) => loc("getByRole", r, o),
    getByLabel: (a: unknown) => loc("getByLabel", a),
    getByPlaceholder: (a: unknown) => loc("getByPlaceholder", a),
    getByTestId: (a: unknown) => loc("getByTestId", a),
    getByText: (a: unknown) => loc("getByText", a),
    locator: (a: unknown) => loc("locator", a),
  } as unknown as Page;
  const firstCall = (fn: string): Matchers => calls.find((c) => c.fn === fn)?.opts ?? {};
  return { page, calls, firstCall };
}

describe("autoCandidates", () => {
  it("tries exact interactive-role matches first and bare text last", () => {
    const { page, calls } = stubPage();
    autoCandidates(page, "Save");
    const fns = calls.map((c) => c.fn);
    const lastRole = fns.lastIndexOf("getByRole");
    expect(lastRole >= 0 && lastRole < fns.indexOf("locator.filter")).toBe(true);
    const last = calls.at(-1);
    expect(last?.fn).toBe("getByText");
    expect(last?.arg instanceof RegExp && last.arg.test("Save changes")).toBe(true);
  });

  it("probes button and link before the rarer roles", () => {
    const { page, calls } = stubPage();
    autoCandidates(page, "Go");
    expect(calls.slice(0, 2).map((c) => [c.fn, c.arg])).toEqual([
      ["getByRole", "button"],
      ["getByRole", "link"],
    ]);
  });

  it("matches an exact name but not a longer one that contains it", () => {
    const { page, firstCall } = stubPage();
    autoCandidates(page, "Save");
    const exact = firstCall("getByRole");
    expect(exact.name?.test("Save")).toBe(true);
    expect(exact.name?.test("save"), "names are case-insensitive").toBe(true);
    expect(exact.name && !exact.name.test("Save and close")).toBe(true);
  });

  it("matches a name embedded in a longer row on the contains rung", () => {
    const { page, firstCall } = stubPage();
    autoCandidates(page, "AGENTS.md");
    const filtered = firstCall("locator.filter");
    expect(filtered.hasText?.test("AGENTS.md Markdown · empty Empty file")).toBe(true);
  });

  it("escapes regex metacharacters in names", () => {
    const { page, firstCall } = stubPage();
    autoCandidates(page, "AGENTS.md");
    const exact = firstCall("getByRole");
    expect(exact.name?.test("AGENTS.md")).toBe(true);
    expect(exact.name && !exact.name.test("AGENTSXmd")).toBe(true);
  });
});
