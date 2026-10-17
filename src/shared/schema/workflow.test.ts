import { describe, expect, it } from "vitest";
import { defaultMix, stripSchema } from "../mix/types";
import { GraphBuilder } from "../templates/build";
import { normalizeWorkflow } from "./workflow";

describe("normalizeWorkflow", () => {
  it("prunes mixer channels of deleted nodes", () => {
    const g = new GraphBuilder();
    g.node("m", "audio.music", 0, 0);
    const wf = g.build({ id: "x", name: "x" });
    const mix = defaultMix();
    mix.channels.gone = stripSchema.parse({ fader: -3 });
    mix.channels.m = stripSchema.parse({ fader: -6 });
    expect(Object.keys(normalizeWorkflow({ ...wf, mix }).mix?.channels ?? {})).toEqual(["m"]);
  });
});
