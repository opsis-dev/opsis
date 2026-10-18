import { describe, expect, it } from "vitest";
import type { Locator, Page } from "playwright-core";
import { CancelledError } from "../../shared/errors";
import { resolveTarget, SelectorError } from "./selectors";

type FakeLocator = Pick<Locator, "filter" | "locator" | "first" | "isVisible" | "waitFor">;

type FakePage = Pick<
  Page,
  | "getByRole"
  | "getByLabel"
  | "getByPlaceholder"
  | "getByTestId"
  | "getByText"
  | "locator"
  | "waitForTimeout"
  | "evaluate"
>;

const asLocator = (fake: FakeLocator): Locator => fake as unknown as Locator;

const asPage = (fake: FakePage): Page => fake as unknown as Page;

function fakePage(shown: (query: string) => boolean) {
  const waited: string[] = [];
  let polls = 0;
  const locator = (query: string): Locator =>
    asLocator({
      filter: (opts) => locator(`${query}.filter(${String(opts?.hasText)})`),
      locator: (selector) => locator(`${query}.locator(${String(selector)})`),
      first: () => locator(query),
      isVisible: async () => shown(query),
      waitFor: async () => void waited.push(query),
    });
  const page = asPage({
    getByRole: (role, opts) => locator(`role ${role} ${String(opts?.name)}`),
    getByLabel: (text) => locator(`label ${String(text)}`),
    getByPlaceholder: (text) => locator(`placeholder ${String(text)}`),
    getByTestId: (id) => locator(`testid ${String(id)}`),
    getByText: (text) => locator(`text ${String(text)}`),
    locator: (selector) => locator(`css ${String(selector)}`),
    waitForTimeout: async () => void polls++,
    evaluate: async () => {
      throw new Error("Nothing to list.");
    },
  });
  return { page, waited, polls: () => polls };
}

const opts = (more: Partial<Parameters<typeof resolveTarget>[2]> = {}) => ({
  log: () => undefined,
  signal: new AbortController().signal,
  ...more,
});

describe("resolveTarget", () => {
  it("prefers an exact button over text that merely contains the name", async () => {
    const { page } = fakePage((q) => q.startsWith("role button") || q.startsWith("text"));
    const found = await resolveTarget(page, "Save", opts());
    expect(found.by).toBe('role=button name="Save"');
  });

  it("falls through the roles to the first rung that shows something", async () => {
    const { page } = fakePage((q) => q.startsWith("label"));
    expect((await resolveTarget(page, "Email", opts())).by).toBe('label="Email"');
  });

  it("keeps looking until the element appears", async () => {
    let checks = 0;
    const { page, polls } = fakePage((q) => q.startsWith("testid") && ++checks > 1);
    const found = await resolveTarget(page, "cart", opts());
    expect(found.by).toBe('testid="cart"');
    expect(polls()).toBe(1);
  });

  it("uses a held element, a point or a selector as given", async () => {
    const { page, waited } = fakePage(() => false);
    const held = asLocator({
      filter: () => held,
      locator: () => held,
      first: () => held,
      isVisible: async () => true,
      waitFor: async () => undefined,
    });
    const hoisted = new Map([["Row", held]]);
    expect(await resolveTarget(page, "Row", opts({ hoisted }))).toEqual({
      target: held,
      by: "hoisted",
    });
    expect(await resolveTarget(page, "120, 48", opts())).toEqual({
      target: { x: 120, y: 48 },
      by: "point",
    });
    expect((await resolveTarget(page, "css=#add", opts())).by).toBe("css=#add");
    expect(waited).toEqual(["css #add"]);
  });

  it("lists what it tried when nothing turns up in time", async () => {
    const { page } = fakePage(() => false);
    const error = await resolveTarget(page, "Missing", opts({ timeoutMs: 0 })).then(
      () => null,
      (e: unknown) => e,
    );
    if (!(error instanceof SelectorError)) throw new Error("Expected the element to be missing.");
    expect(error.tried[0]).toBe('role=button name="Missing"');
    expect(error.tried.at(-1)).toBe('text "Missing"');
  });

  it("stops when the take is cancelled", async () => {
    const { page } = fakePage(() => false);
    const controller = new AbortController();
    controller.abort();
    await expect(
      resolveTarget(page, "Save", opts({ signal: controller.signal })),
    ).rejects.toBeInstanceOf(CancelledError);
  });
});
