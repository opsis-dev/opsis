export const LANE_GAP = 14;
const STEP_OFFSET = 20;

export type LaneBox = { x: number; y: number; width: number; height: number };

export type LaneEdge = { id: string; source: string; target: string };

type Run = { id: string; column: number; left: number; right: number; top: number; bottom: number };

function runOf(edge: LaneEdge, boxes: ReadonlyMap<string, LaneBox>): Run | null {
  const from = boxes.get(edge.source);
  const to = boxes.get(edge.target);
  if (!from || !to) return null;
  const left = from.x + from.width + STEP_OFFSET;
  const right = to.x - STEP_OFFSET;
  if (right <= left) return null;
  return {
    id: edge.id,
    column: Math.round(to.x),
    left,
    right,
    top: Math.min(from.y, to.y),
    bottom: Math.max(from.y + from.height, to.y + to.height),
  };
}

function lanesOf(runs: Run[]): Map<string, number> {
  const ordered = runs.toSorted(
    (a, b) => a.top - b.top || a.bottom - b.bottom || a.id.localeCompare(b.id),
  );
  const ends: number[] = [];
  const lanes = new Map<string, number>();
  for (const run of ordered) {
    let lane = ends.findIndex((end) => end < run.top);
    if (lane === -1) lane = ends.length;
    ends[lane] = run.bottom;
    lanes.set(run.id, lane);
  }
  return lanes;
}

export function laneCentres(
  boxes: ReadonlyMap<string, LaneBox>,
  edges: readonly LaneEdge[],
): Map<string, { x: number; left: number; right: number }> {
  const columns = new Map<number, Run[]>();
  for (const edge of edges) {
    const run = runOf(edge, boxes);
    if (!run) continue;
    columns.set(run.column, [...(columns.get(run.column) ?? []), run]);
  }
  const centres = new Map<string, { x: number; left: number; right: number }>();
  for (const runs of columns.values()) {
    const lanes = lanesOf(runs);
    const count = Math.max(...lanes.values()) + 1;
    const left = Math.max(...runs.map((r) => r.left));
    const right = Math.min(...runs.map((r) => r.right));
    if (right <= left) continue;
    const spacing = Math.min(LANE_GAP, (right - left) / (count + 1));
    const middle = (left + right) / 2;
    for (const run of runs) {
      const lane = lanes.get(run.id) ?? 0;
      const x = middle + (lane - (count - 1) / 2) * spacing;
      centres.set(run.id, { x, left: run.left, right: run.right });
    }
  }
  return centres;
}

export function stepPositions(
  boxes: ReadonlyMap<string, LaneBox>,
  edges: readonly LaneEdge[],
): Map<string, number> {
  return new Map(
    [...laneCentres(boxes, edges)].map(([id, { x, left, right }]) => [
      id,
      (x - left) / (right - left),
    ]),
  );
}
