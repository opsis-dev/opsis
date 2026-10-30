import { describe, expect, it, vi } from "vitest";
import { createCutBoard } from "#video/core/transition-cuts.tsx";

function image() {
  return {} as OffscreenCanvas;
}

describe("cut board", () => {
  it("draws both scenes through the incoming side once both report the same progress", () => {
    const board = createCutBoard();
    const incoming = { image: image(), progress: 0.4, draw: vi.fn() };
    const outgoing = { image: image(), progress: 0.4, draw: vi.fn() };
    board.report(30, "exiting", outgoing);
    board.report(30, "entering", incoming);
    expect(incoming.draw).toHaveBeenLastCalledWith(incoming.image, outgoing.image, 0.4);
    expect(outgoing.draw).toHaveBeenLastCalledWith(null, null, 0);
  });

  it("waits while the two sides are a frame apart", () => {
    const board = createCutBoard();
    const outgoing = { image: image(), progress: 0.5, draw: vi.fn() };
    const incoming = { image: image(), progress: 0.4, draw: vi.fn() };
    board.report(30, "exiting", outgoing);
    outgoing.draw.mockClear();
    board.report(30, "entering", incoming);
    expect(incoming.draw).not.toHaveBeenCalled();
    expect(outgoing.draw).not.toHaveBeenCalled();
  });

  it("draws a side on its own when nothing is on the other side of the cut", () => {
    const board = createCutBoard();
    const incoming = { image: image(), progress: 0.2, draw: vi.fn() };
    board.report(0, "entering", incoming);
    expect(incoming.draw).toHaveBeenLastCalledWith(incoming.image, null, 0.2);
    const outgoing = { image: image(), progress: 0.7, draw: vi.fn() };
    board.report(90, "exiting", outgoing);
    expect(outgoing.draw).toHaveBeenLastCalledWith(null, outgoing.image, 0.7);
  });

  it("keeps cuts apart and draws the side that stays when the other leaves", () => {
    const board = createCutBoard();
    const outgoing = { image: image(), progress: 0.5, draw: vi.fn() };
    const incoming = { image: image(), progress: 0.5, draw: vi.fn() };
    const elsewhere = { image: image(), progress: 0.5, draw: vi.fn() };
    board.report(30, "exiting", outgoing);
    board.report(30, "entering", incoming);
    board.report(60, "entering", elsewhere);
    expect(elsewhere.draw).toHaveBeenLastCalledWith(elsewhere.image, null, 0.5);
    board.leave(30, "exiting");
    expect(incoming.draw).toHaveBeenLastCalledWith(incoming.image, null, 0.5);
  });
});
