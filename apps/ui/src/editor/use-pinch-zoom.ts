import { useReactFlow } from "@xyflow/react";
import { useEffect } from "react";
import { pinchViewport } from "./pinch";

export function usePinchZoom(
  wrapper: React.RefObject<HTMLDivElement | null>,
  enabled: boolean,
): void {
  const flow = useReactFlow();
  useEffect(() => {
    const el = wrapper.current;
    if (!el || !enabled) return;
    const onWheel = (e: WheelEvent) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      e.stopPropagation();
      const box = el.getBoundingClientRect();
      const current = flow.getViewport();
      const next = pinchViewport(current, e.deltaY, e.clientX - box.left, e.clientY - box.top);
      if (next.zoom !== current.zoom) void flow.setViewport(next);
    };
    el.addEventListener("wheel", onWheel, { capture: true, passive: false });
    return () => el.removeEventListener("wheel", onWheel, { capture: true });
  }, [wrapper, flow, enabled]);
}
