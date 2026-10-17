import { useReactFlow } from "@xyflow/react";
import { useEffect } from "react";
import { useUiStore } from "../store/ui";

const FALLBACK_SIZE = { width: 260, height: 120 };

export function useRevealNode(wrapper: React.RefObject<HTMLDivElement | null>): void {
  const flow = useReactFlow();
  const reveal = useUiStore((s) => s.reveal);
  useEffect(() => {
    if (!reveal) return;
    const node = flow.getNode(reveal.id);
    const internal = flow.getInternalNode(reveal.id);
    const box = wrapper.current?.getBoundingClientRect();
    if (!node || !internal || !box) return;
    const { x, y, zoom } = flow.getViewport();
    const abs = internal.internals.positionAbsolute;
    const left = abs.x * zoom + x;
    const top = abs.y * zoom + y;
    const width = (internal.measured.width ?? FALLBACK_SIZE.width) * zoom;
    const height = (internal.measured.height ?? FALLBACK_SIZE.height) * zoom;
    if (left >= 0 && top >= 0 && left + width <= box.width && top + height <= box.height) return;
    const target = node.parentId ? [{ id: node.parentId }, { id: node.id }] : [{ id: node.id }];
    void flow.fitView({
      nodes: target,
      duration: 300,
      maxZoom: Math.max(flow.getZoom(), 0.8),
      padding: 0.4,
    });
  }, [reveal, flow, wrapper]);
}
