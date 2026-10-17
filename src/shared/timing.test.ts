import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { debounce, oneAtATime } from "./timing";

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

describe("oneAtATime", () => {
  it("starts each job after the one before it settles, even when it fails", async () => {
    const serial = oneAtATime();
    const order: string[] = [];
    const slow = serial(async () => {
      await new Promise((resolve) => setTimeout(resolve, 5));
      order.push("first");
      throw new Error("failed");
    });
    const second = serial(async () => {
      order.push("second");
      return 2;
    });
    await expect(slow).rejects.toThrow("failed");
    await expect(second).resolves.toBe(2);
    expect(order).toEqual(["first", "second"]);
  });
});
