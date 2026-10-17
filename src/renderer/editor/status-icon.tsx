import { Circle, CircleCheck, CircleSlash, CircleX, History, type LucideIcon } from "lucide-react";
import type { NodeRunStatus } from "@/shared/schema/events";
import { Spinner } from "@/renderer/components/ui/spinner";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { cn } from "@/renderer/lib/utils";

export type RowState = NodeRunStatus | "failedBefore" | "fromLastRun";

const STATUS_ICONS: Record<RowState, { icon: LucideIcon; word: string; className: string }> = {
  idle: { icon: Circle, word: "Not run", className: "text-muted-foreground" },
  queued: { icon: Circle, word: "Waiting", className: "text-muted-foreground" },
  running: { icon: Circle, word: "Running", className: "" },
  done: { icon: CircleCheck, word: "Done", className: "text-success" },
  cached: { icon: CircleCheck, word: "Up to date", className: "text-muted-foreground" },
  skipped: { icon: CircleSlash, word: "Skipped", className: "text-muted-foreground" },
  error: { icon: CircleX, word: "Failed", className: "text-destructive" },
  failedBefore: { icon: CircleX, word: "Failed last time", className: "text-destructive/60" },
  fromLastRun: { icon: History, word: "From last run", className: "text-muted-foreground" },
};

export function StatusIcon({ state }: { state: RowState }) {
  const { icon: Icon, word, className } = STATUS_ICONS[state];
  return (
    <Tooltip>
      <TooltipTrigger
        render={<span className="flex size-4 shrink-0 items-center justify-center" />}
      >
        {state === "running" ? (
          <Spinner aria-label={word} />
        ) : (
          <Icon aria-label={word} className={cn("size-4", className)} />
        )}
      </TooltipTrigger>
      <TooltipContent>{word}</TooltipContent>
    </Tooltip>
  );
}
