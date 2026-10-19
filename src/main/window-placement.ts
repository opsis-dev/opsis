import { screen, type Rectangle } from "electron";

export const SAVE_BOUNDS_AFTER_MS = 400;
const MIN_VISIBLE_PX = 40;
const ABOVE_WORK_AREA_PX = 20;

function isVisible(rect: Rectangle): boolean {
  return screen.getAllDisplays().some(({ workArea: area }) => {
    const overlapsAcross =
      rect.x < area.x + area.width - MIN_VISIBLE_PX &&
      rect.x + rect.width > area.x + MIN_VISIBLE_PX;
    const titleBarReachable =
      rect.y >= area.y - ABOVE_WORK_AREA_PX && rect.y < area.y + area.height - MIN_VISIBLE_PX;
    return overlapsAcross && titleBarReachable;
  });
}

export const visibleBounds = (rect: Rectangle | null | undefined): Rectangle | undefined =>
  rect && isVisible(rect) ? rect : undefined;
