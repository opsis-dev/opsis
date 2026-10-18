import { describe, expect, it, vi } from "vitest";
import { GraphBuilder } from "@/shared/templates/build";

const workflow = new GraphBuilder().build({ name: "Ids" });

async function firstIdOfALaunch(): Promise<string> {
  vi.resetModules();
  const { editContext, gestureSalt } = await import("./timeline-drag");
  return editContext(workflow, 30, null, gestureSalt()).makeId("timeline");
}

describe("timeline edit ids", () => {
  it("never repeat an id minted in an earlier launch", async () => {
    expect(await firstIdOfALaunch()).not.toBe(await firstIdOfALaunch());
  });
});
