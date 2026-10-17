import { useReactFlow } from "@xyflow/react";
import { useEffect } from "react";
import { useUiStore } from "../store/ui";
import { cardSize } from "./card-size";
import type { CanvasNode } from "./convert";

const REVEAL_MS = 300;
const MIN_REVEAL_ZOOM = 0.8;
const REVEAL_PADDING = 0.4;

export function useRevealNode(wrapper: React.RefObject<HTMLDivElement | null>): void {
  const flow = useReactFlow<CanvasNode>();
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
    const size =
      internal.measured.width && internal.measured.height
        ? { width: internal.measured.width, height: internal.measured.height }
        : cardSize(node.data.node);
    const width = size.width * zoom;
    const height = size.height * zoom;
    if (left >= 0 && top >= 0 && left + width <= box.width && top + height <= box.height) return;
    const target = node.parentId ? [{ id: node.parentId }, { id: node.id }] : [{ id: node.id }];
    void flow.fitView({
      nodes: target,
      duration: REVEAL_MS,
      maxZoom: Math.max(flow.getZoom(), MIN_REVEAL_ZOOM),
      padding: REVEAL_PADDING,
    });
  }, [reveal, flow, wrapper]);
}
