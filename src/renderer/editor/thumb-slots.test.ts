import { describe, expect, it } from "vitest";
import { claimSlot } from "./thumb-slots";

const FULL = 6;

const plain = (count: number, prefix = "p") =>
  Array.from({ length: count }, (_, i) => ({ id: `${prefix}${i}`, priority: false }));

describe("claimSlot", () => {
  it("adds a holder while there is room and keeps one that already holds", () => {
    const active = plain(1);
    expect(claimSlot(active, "b", false)).toEqual([...active, { id: "b", priority: false }]);
    expect(claimSlot(active, "p0", true)).toBe(active);
  });

  it("refuses a plain claim when full", () => {
    const active = plain(FULL);
    expect(claimSlot(active, "c", false)).toBe(active);
  });

  it("evicts the oldest plain holder for a priority claim", () => {
    const active = [{ id: "a", priority: true }, ...plain(FULL - 1)];
    expect(claimSlot(active, "d", true)).toEqual([
      { id: "a", priority: true },
      ...plain(FULL - 1).slice(1),
      { id: "d", priority: true },
    ]);
  });

  it("evicts the oldest holder when every holder has priority", () => {
    const active = plain(FULL).map((s) => ({ ...s, priority: true }));
    expect(claimSlot(active, "c", true)).toEqual([...active.slice(1), { id: "c", priority: true }]);
  });
});
