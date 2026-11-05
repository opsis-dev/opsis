import type { InternalNode } from "@xyflow/react";
import * as fc from "fast-check";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { setViewProject } from "../store/view-prefs";
import { cardShape, loadCardShapes, rememberCardShapes, saveCardShapesNow } from "./card-shapes";
import { measuredPorts, type MeasuredPort } from "./card-size";

type Bound = { id: string; type: "source" | "target"; y: number };

function measuredNode(id: string, width: number, height: number, bounds: Bound[]): InternalNode {
  function handle(b: Bound) {
    return {
      id: b.id,
      nodeId: id,
      type: b.type,
      position: b.type === "target" ? "left" : "right",
      x: b.type === "target" ? 0 : width,
      y: b.y,
      width: 8,
      height: 8,
    };
  }
  return {
    id,
    measured: { width, height },
    internals: {
      handleBounds: {
        source: bounds.filter((b) => b.type === "source").map(handle),
        target: bounds.filter((b) => b.type === "target").map(handle),
      },
    },
  } as unknown as InternalNode;
}

function byHandle(ports: readonly MeasuredPort[]): MeasuredPort[] {
  return ports.toSorted((a, b) => a.handle.localeCompare(b.handle));
}

const card = fc.record({
  id: fc.uuid(),
  width: fc.integer({ min: 1, max: 2000 }),
  height: fc.integer({ min: 1, max: 2000 }),
  kept: fc.boolean(),
  bounds: fc.uniqueArray(
    fc.record({
      id: fc.stringMatching(/^[a-z]{1,8}$/),
      type: fc.constantFrom<"source" | "target">("source", "target"),
      y: fc.integer({ min: 0, max: 2000 }),
    }),
    { selector: (b) => b.id, maxLength: 6 },
  ),
});

describe("card shapes", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    setViewProject(null, {});
    loadCardShapes();
  });
  afterEach(() => vi.useRealTimers());

  it("saves the measured shapes of the nodes still there and loads them back as they were", () => {
    fc.assert(
      fc.property(fc.uniqueArray(card, { selector: (c) => c.id, maxLength: 12 }), (cards) => {
        setViewProject(null, {});
        loadCardShapes();
        rememberCardShapes(
          cards.map((c) => measuredNode(c.id, c.width, c.height, c.bounds)),
          new Set(cards.filter((c) => c.kept).map((c) => c.id)),
        );
        saveCardShapesNow();
        loadCardShapes();
        const loaded = cards.map((c) => {
          const shape = cardShape(c.id);
          return (
            shape && {
              width: shape.width,
              height: shape.height,
              ports: byHandle(measuredPorts(c.id) ?? []),
            }
          );
        });
        const expected = cards.map((c) =>
          c.kept
            ? {
                width: c.width,
                height: c.height,
                ports: byHandle(
                  c.bounds.map((b) => ({
                    handle: b.id,
                    side: b.type === "target" ? ("in" as const) : ("out" as const),
                    y: b.y + 4,
                  })),
                ),
              }
            : undefined,
        );
        expect(loaded).toEqual(expected);
      }),
    );
  });

  it("waits a second after the last measurement before saving", () => {
    rememberCardShapes([measuredNode("a", 200, 100, [])], new Set(["a"]));
    loadCardShapes();
    expect(cardShape("a")).toBeUndefined();
    rememberCardShapes([measuredNode("a", 200, 100, [])], new Set(["a"]));
    vi.advanceTimersByTime(1000);
    loadCardShapes();
    expect(cardShape("a")).toEqual({ width: 200, height: 100, handles: [] });
  });
});
