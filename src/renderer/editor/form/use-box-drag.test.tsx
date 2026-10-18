import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Bounds } from "@/shared/schema/actions";
import { boxHandle, useBoxDrag } from "./use-box-drag";

afterEach(cleanup);

const SIZE = 100;

function Boxes({ boxes, onChange }: { boxes: Bounds[]; onChange: (b: Bounds[]) => void }) {
  const { area, frame } = useBoxDrag(boxes, onChange, true);
  return (
    <div ref={area} data-testid="frame" {...frame}>
      {boxes.map((_, i) => (
        <div key={i} data-testid={`box${i}`} {...boxHandle(i)} />
      ))}
    </div>
  );
}

function drawn(boxes: Bounds[], from: string, path: Array<[number, number]>): Bounds[] {
  let changed = boxes;
  const view = render(<Boxes boxes={boxes} onChange={(b) => (changed = b)} />);
  const frame = view.getByTestId("frame");
  frame.getBoundingClientRect = () => new DOMRect(0, 0, SIZE, SIZE);
  const captured = new Set<number>();
  frame.setPointerCapture = (id) => void captured.add(id);
  frame.hasPointerCapture = (id) => captured.has(id);
  const [[x0, y0], ...rest] = path;
  fireEvent.pointerDown(view.getByTestId(from), {
    pointerId: 1,
    button: 0,
    clientX: x0,
    clientY: y0,
  });
  for (const [x, y] of rest) fireEvent.pointerMove(frame, { pointerId: 1, clientX: x, clientY: y });
  fireEvent.lostPointerCapture(frame, { pointerId: 1 });
  return changed;
}

describe("useBoxDrag", () => {
  it("draws a new area across the frame", () => {
    expect(
      drawn([], "frame", [
        [10, 10],
        [50, 40],
      ]),
    ).toEqual([{ x: 0.1, y: 0.1, width: 0.4, height: 0.3 }]);
  });

  it("moves an area grabbed by its handle", () => {
    const box = { x: 0.1, y: 0.1, width: 0.2, height: 0.2 };
    expect(
      drawn([box], "box0", [
        [15, 15],
        [35, 25],
      ]),
    ).toEqual([{ ...box, x: 0.3, y: 0.2 }]);
  });
});
