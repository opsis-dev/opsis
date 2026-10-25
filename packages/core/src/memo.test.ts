import { describe, expect, it } from "vitest";
import { keepLatest, lruCache, shareInFlight } from "#core/memo";

describe("shareInFlight", () => {
  it("runs one job per key at a time and starts again once it settles", async () => {
    const share = shareInFlight<number>();
    let runs = 0;
    const job = () => Promise.resolve(++runs);
    const [a, b] = await Promise.all([share("k", job), share("k", job)]);
    expect([a, b, runs]).toEqual([1, 1, 1]);
    expect(await share("k", job)).toBe(2);
    expect(await share("other", job)).toBe(3);
  });

  it("lets a failed job be tried again", async () => {
    const share = shareInFlight<number>();
    await expect(share("k", () => Promise.reject(new Error("boom")))).rejects.toThrow("boom");
    expect(await share("k", () => Promise.resolve(7))).toBe(7);
  });
});

describe("lruCache", () => {
  it("makes a value once, keeps recently used ones and drops the oldest past the limit", () => {
    const cache = lruCache<number>(2);
    let made = 0;
    const make = () => ++made;
    expect(cache.use("a", make)).toBe(1);
    expect(cache.use("b", make)).toBe(2);
    expect(cache.use("a", make)).toBe(1);
    expect(cache.use("c", make)).toBe(3);
    expect(cache.peek("a")).toBe(1);
    expect(cache.peek("b")).toBeUndefined();
    cache.delete("a");
    expect(cache.use("a", make)).toBe(4);
  });
});

describe("keepLatest", () => {
  it("keeps one value, reuses it for the same key and disposes of it when the key changes", async () => {
    const disposed: string[] = [];
    const latest = keepLatest<string>((value) => disposed.push(value));
    let made = 0;
    const make = (name: string) => () => Promise.resolve(`${name}${++made}`);
    expect(await latest("a", make("a"))).toBe("a1");
    expect(await latest("a", make("a"))).toBe("a1");
    expect(await latest("b", make("b"))).toBe("b2");
    await Promise.resolve();
    expect(disposed).toEqual(["a1"]);
  });

  it("forgets a value that failed, so the same key can be made again", async () => {
    const latest = keepLatest<number>();
    await expect(latest("k", () => Promise.reject(new Error("boom")))).rejects.toThrow("boom");
    expect(await latest("k", () => Promise.resolve(3))).toBe(3);
  });
});
