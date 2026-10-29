import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { MouseEvent } from "react";
import { commands } from "@/generated/bindings";
import { DRAG_START_PX } from "@/lib/dom";

const CONTROLS = [
  "button",
  "a",
  "input",
  "textarea",
  "select",
  "label",
  "summary",
  "[role='button']",
  "[role='link']",
  "[role='menuitem']",
  "[role='tab']",
  "[role='checkbox']",
  "[role='radio']",
  "[role='switch']",
  "[role='option']",
  "[contenteditable='true']",
  "[data-no-drag]",
].join(", ");

const onControl = (e: MouseEvent): boolean =>
  e.target instanceof Element && e.target.closest(CONTROLS) !== null;

function dragOnceMoved(x0: number, y0: number): void {
  const stop = () => {
    window.removeEventListener("mousemove", move);
    window.removeEventListener("mouseup", stop);
  };
  const move = (e: globalThis.MouseEvent) => {
    if (Math.hypot(e.clientX - x0, e.clientY - y0) < DRAG_START_PX) return;
    stop();
    void getCurrentWebviewWindow().startDragging();
  };
  window.addEventListener("mousemove", move);
  window.addEventListener("mouseup", stop);
}

export const windowDragHandlers = {
  onMouseDown: (e: MouseEvent) => {
    if (e.button !== 0 || e.detail > 1 || onControl(e)) return;
    e.preventDefault();
    dragOnceMoved(e.clientX, e.clientY);
  },
  onDoubleClick: (e: MouseEvent) => {
    if (e.button !== 0 || onControl(e)) return;
    void commands.windowRole("toggleMaximize");
  },
};
