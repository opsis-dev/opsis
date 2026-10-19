import { describe, expect, it } from "vitest";
import { GraphBuilder } from "@/shared/templates/build";
import { editContext } from "./timeline-drag";

const workflow = new GraphBuilder().build({ name: "Ids" });

const idsOf = (salt: string) => {
  const ctx = editContext(workflow, 30, null, salt);
  return [ctx.makeId("timeline"), ctx.makeId("timeline")];
};

describe("timeline edit ids", () => {
  it("are the same for the same salt and never repeat within one edit", () => {
    const ids = idsOf("a");
    expect(idsOf("a")).toEqual(ids);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("differ between edits with different salts", () => {
    const [first] = idsOf("a");
    expect(idsOf("b")).not.toContain(first);
  });
});
