import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { debounce } from "./timing";

describe("debounce", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("runs once with the last arguments after the calls stop", () => {
    const fn = vi.fn();
    const run = debounce(fn, 100);
    run(1);
    vi.advanceTimersByTime(60);
    run(2);
    vi.advanceTimersByTime(99);
    expect(fn).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(fn).toHaveBeenCalledExactlyOnceWith(2);
  });

  it("can be cancelled", () => {
    const fn = vi.fn();
    const run = debounce(fn, 100);
    run();
    run.cancel();
    vi.advanceTimersByTime(200);
    expect(fn).not.toHaveBeenCalled();
  });
});
