import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { listenerSet } from "#core/listener-set";
import { debounce, oneAtATime, whenReady } from "#core/timing";

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

describe("whenReady", () => {
  const source = () => {
    const changes = listenerSet();
    return { changes, subscribe: (fn: () => void) => changes.add(fn) };
  };

  it("resolves with the value once a change makes it ready, and stops listening", async () => {
    const { changes, subscribe } = source();
    let value: string | null = null;
    let listening = 0;
    const ready = whenReady(
      () => value,
      (fn) => {
        listening++;
        const off = subscribe(fn);
        return () => {
          listening--;
          off();
        };
      },
      1000,
    );
    value = "editor";
    changes.emit();
    expect(await ready).toBe("editor");
    expect(listening).toBe(0);
  });

  it("resolves null after the timeout", async () => {
    vi.useFakeTimers();
    const ready = whenReady(() => null, source().subscribe, 1000);
    vi.advanceTimersByTime(1000);
    expect(await ready).toBeNull();
    vi.useRealTimers();
  });
});
