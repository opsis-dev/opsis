export const NODE_DRAG_TYPE = "application/opsis-node";
export const SNAP_GRID = 15;
export const MIN_ZOOM = 0.1;
export const MAX_ZOOM = 2;
export function snapToGrid(p: { x: number; y: number }) {
  return {
    x: Math.round(p.x / SNAP_GRID) * SNAP_GRID,
    y: Math.round(p.y / SNAP_GRID) * SNAP_GRID,
  };
}
