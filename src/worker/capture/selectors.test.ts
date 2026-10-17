import assert from "node:assert/strict";
import { describe, it } from "vitest";
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
  return { page, firstCall };
}

describe("autoCandidates", () => {
  it("puts exact interactive-role matches before any contains match", () => {
    const { page } = stubPage();
    const by = autoCandidates(page, "Save").map((c) => c.by);
    const firstExact = by.findIndex((b) => b.startsWith("role=button"));
    const firstLoose = by.findIndex((b) => b.startsWith("control containing"));
    const bareText = by.findIndex((b) => b === 'text "Save"');
    assert.ok(firstExact >= 0 && firstLoose > firstExact);
    assert.equal(bareText, by.length - 1, "bare text must be the last resort");
  });

  it("probes button and link before the rarer roles", () => {
    const { page } = stubPage();
    const by = autoCandidates(page, "Go").map((c) => c.by);
    assert.equal(by[0], 'role=button name="Go"');
    assert.equal(by[1], 'role=link name="Go"');
  });

  it("matches an exact name but not a longer one that contains it", () => {
    const { page, firstCall } = stubPage();
    autoCandidates(page, "Save");
    const exact = firstCall("getByRole");
    assert.ok(exact.name?.test("Save"));
    assert.ok(exact.name?.test("save"), "names are case-insensitive");
    assert.ok(exact.name && !exact.name.test("Save and close"));
  });

  it("matches a name embedded in a longer row on the contains rung", () => {
    const { page, firstCall } = stubPage();
    autoCandidates(page, "AGENTS.md");
    const filtered = firstCall("locator.filter");
    assert.ok(filtered.hasText?.test("AGENTS.md Markdown · empty Empty file"));
  });

  it("escapes regex metacharacters in names", () => {
    const { page, firstCall } = stubPage();
    autoCandidates(page, "AGENTS.md");
    const exact = firstCall("getByRole");
    assert.ok(exact.name?.test("AGENTS.md"));
    assert.ok(exact.name && !exact.name.test("AGENTSXmd"));
  });

  it("never produces an empty ladder", () => {
    const { page } = stubPage();
    assert.ok(autoCandidates(page, "x").length > 5);
  });
});
