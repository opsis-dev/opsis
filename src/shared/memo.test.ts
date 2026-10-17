import { describe, expect, it } from "vitest";
import { shareInFlight } from "./memo";

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
