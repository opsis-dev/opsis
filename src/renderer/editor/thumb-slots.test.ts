import { describe, expect, it } from "vitest";
import { claimSlot } from "./thumb-slots";

const slots = (...ids: Array<[string, boolean]>) => ids.map(([id, priority]) => ({ id, priority }));

describe("claimSlot", () => {
  it("adds a holder while there is room and keeps one that already holds", () => {
    const active = slots(["a", false]);
    expect(claimSlot(active, "b", false, 2)).toEqual(slots(["a", false], ["b", false]));
    expect(claimSlot(active, "a", true, 2)).toBe(active);
  });

  it("refuses a plain claim when full", () => {
    const active = slots(["a", false], ["b", false]);
    expect(claimSlot(active, "c", false, 2)).toBe(active);
  });

  it("evicts the oldest plain holder for a priority claim", () => {
    const active = slots(["a", true], ["b", false], ["c", false]);
    expect(claimSlot(active, "d", true, 3)).toEqual(slots(["a", true], ["c", false], ["d", true]));
  });

  it("evicts the oldest holder when every holder has priority", () => {
    const active = slots(["a", true], ["b", true]);
    expect(claimSlot(active, "c", true, 2)).toEqual(slots(["b", true], ["c", true]));
  });
});
