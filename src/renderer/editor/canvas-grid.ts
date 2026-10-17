export const NODE_DRAG_TYPE = "application/demoflow-node";
export const SNAP_GRID = 15;
export const snapToGrid = (p: { x: number; y: number }) => ({
  x: Math.round(p.x / SNAP_GRID) * SNAP_GRID,
  y: Math.round(p.y / SNAP_GRID) * SNAP_GRID,
});
