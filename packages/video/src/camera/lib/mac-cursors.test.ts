import { expect, it } from "vitest";
import {
  MAC_CURSOR_BOX,
  MAC_CURSOR_KEYWORDS,
  MAC_CURSORS,
} from "#video/camera/lib/mac-cursors.generated.ts";

it("draws every cursor keyword with a cursor the set has", () => {
  for (const name of Object.values(MAC_CURSOR_KEYWORDS)) expect(MAC_CURSORS[name]).toBeDefined();
});

it("keeps every hotspot inside the artwork", () => {
  for (const { hotspot } of Object.values(MAC_CURSORS))
    for (const at of hotspot) expect(at >= 0 && at <= MAC_CURSOR_BOX).toBe(true);
});
