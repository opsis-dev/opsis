import type { Workflow } from "@opsis/core/schema/workflow";
import { SquareChartGantt } from "lucide-react";
import { useRef } from "react";
import { isEditingElement } from "@/lib/dom";
import { EmptyPanel } from "../../app/empty-panel";
import { useWorkflowStore } from "../../store/workflow";
import { useFilm, type Film } from "../film";
import { focusTargetRef } from "../panels";
import { RemoveDialog } from "./remove-dialog";
import { timelineKeyAction } from "./timeline-keys";
import { TimelineToolbar } from "./timeline-toolbar";
import { TimelineView, useScrollerView, type TimelineFilm } from "./timeline-view";
import { useTimelineCommands } from "./use-timeline-commands";
import { useTimelineZoom } from "./use-timeline-zoom";

const hasFlat = (film: Film): film is TimelineFilm => film.flat !== null;

const timelineFocus = focusTargetRef("timeline");

function TimelineEditor({ film, workflow }: { film: TimelineFilm; workflow: Workflow }) {
  const { flat, sourceNodeId } = film;
  const fps = flat.fps;
  const duration = flat.durationInFrames;
  const scroller = useRef<HTMLDivElement>(null);
  const scroll = useScrollerView(scroller);
  const zoom = useTimelineZoom(scroller, scroll.view.width, duration);
  const { commands, removal, confirmRemoval, closeRemoval } = useTimelineCommands({
    workflow,
    flat,
    sourceNodeId,
    zoom,
  });

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (isEditingElement(e.target)) return;
    const action = timelineKeyAction(e);
    if (!action) return;
    action({ commands, fps, duration, shift: e.shiftKey });
    e.preventDefault();
    e.stopPropagation();
  };

  return (
    <div
      ref={timelineFocus}
      data-timeline-root
      tabIndex={0}
      className="flex h-full flex-col outline-none"
      onKeyDown={onKeyDown}
    >
      <TimelineToolbar
        commands={commands}
        workflow={workflow}
        sourceNodeId={sourceNodeId}
        fps={fps}
        duration={duration}
      />
      <div className="min-h-0 flex-1">
        <TimelineView
          scroller={scroller}
          scroll={scroll}
          zoom={zoom}
          film={film}
          workflow={workflow}
          commands={commands}
        />
      </div>
      <RemoveDialog removal={removal} onConfirm={confirmRemoval} onClose={closeRemoval} />
    </div>
  );
}

export function TimelinePanel() {
  const film = useFilm();
  const workflow = useWorkflowStore((s) => s.workflow);

  if (!hasFlat(film) || !workflow) {
    return (
      <EmptyPanel
        icon={SquareChartGantt}
        title="No film to show yet"
        description="Connect clips to an export, or select a node that makes a clip."
      />
    );
  }

  return <TimelineEditor film={film} workflow={workflow} />;
}
