import { describe, expect, it, vi } from "vitest";
import { onAbort } from "./errors";

describe("onAbort", () => {
  it("runs once on abort and not after it is stopped", () => {
    const first = new AbortController();
    const fn = vi.fn();
    onAbort(first.signal, fn);
    first.abort();
    first.abort();
    expect(fn).toHaveBeenCalledTimes(1);

    const second = new AbortController();
    const stopped = vi.fn();
    onAbort(second.signal, stopped)();
    second.abort();
    expect(stopped).not.toHaveBeenCalled();
  });

  it("does nothing without a signal", () => {
    expect(() => onAbort(undefined, () => undefined)()).not.toThrow();
  });
});
