import { expect, it } from "vitest";
import { pageTurnLift } from "#video/elements/transition/page-turn.tsx";

function lowestAfterTurn(width: number, height: number, angle: number) {
  const theta = (angle * Math.PI) / 180;
  const lift = pageTurnLift(width, height, 0.18, angle);
  const corners = [
    [0, 0],
    [width, 0],
    [0, height],
    [width, height],
  ];
  return Math.max(
    ...corners.map(
      ([x, y]) =>
        height + (x - 0.18 * width) * Math.sin(theta) + (y - height) * Math.cos(theta) - lift,
    ),
  );
}

it("lifts the turned page clear of the frame at every angle and shape", () => {
  for (const [width, height] of [
    [1920, 1080],
    [1080, 1920],
    [1080, 1080],
  ])
    for (let angle = -20; angle <= 20; angle += 5)
      expect(lowestAfterTurn(width, height, angle)).toBeLessThanOrEqual(0);
});
