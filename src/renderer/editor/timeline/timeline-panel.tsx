import { SquareChartGantt } from "lucide-react";
import { useRef } from "react";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/renderer/components/ui/empty";
import { isEditingElement } from "@/renderer/lib/dom";
import { useWorkflowStore } from "../../store/workflow";
import { useFilm } from "../film";
import { timelineKeyAction } from "./timeline-keys";
import { TimelineToolbar } from "./timeline-toolbar";
import { TimelineView, type TimelineActions } from "./timeline-view";

export function TimelinePanel() {
  const film = useFilm();
  const workflow = useWorkflowStore((s) => s.workflow);
  const actions = useRef<TimelineActions | null>(null);

  if (!film.flat || !workflow) {
    return (
      <Empty className="h-full">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <SquareChartGantt />
          </EmptyMedia>
          <EmptyTitle>No film to show yet</EmptyTitle>
          <EmptyDescription>
            Connect clips to an export, or select a node that makes a clip.
          </EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  }

  const fps = film.flat.fps;
  const duration = film.flat.durationInFrames;

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (isEditingElement(e.target)) return;
    const action = timelineKeyAction(e);
    if (!action) return;
    action({ actions: actions.current, fps, duration, shift: e.shiftKey });
    e.preventDefault();
    e.stopPropagation();
  };

  return (
    <div
      data-timeline-root
      tabIndex={0}
      className="flex h-full flex-col outline-none"
      onKeyDown={onKeyDown}
    >
      <TimelineToolbar
        actions={actions}
        workflow={workflow}
        sourceNodeId={film.sourceNodeId}
        fps={fps}
        duration={duration}
      />
      <div className="min-h-0 flex-1">
        <TimelineView ref={actions} workflow={workflow} flat={film.flat} />
      </div>
    </div>
  );
}
