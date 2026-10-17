import { Circle, CircleCheck, CircleSlash, CircleX, History, type LucideIcon } from "lucide-react";
import type { NodeRunStatus } from "@/shared/schema/events";
import { Spinner } from "@/renderer/components/ui/spinner";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { cn } from "@/renderer/lib/utils";

export type RowState = NodeRunStatus | "failedBefore" | "fromLastRun";

const RUNNING_WORD = "Running";

const STATUS_ICONS: Record<
  Exclude<RowState, "running">,
  { icon: LucideIcon; word: string; className: string }
> = {
  idle: { icon: Circle, word: "Not run", className: "text-muted-foreground" },
  queued: { icon: Circle, word: "Waiting", className: "text-muted-foreground" },
  done: { icon: CircleCheck, word: "Done", className: "text-success" },
  cached: { icon: CircleCheck, word: "Up to date", className: "text-muted-foreground" },
  skipped: { icon: CircleSlash, word: "Skipped", className: "text-muted-foreground" },
  error: { icon: CircleX, word: "Failed", className: "text-destructive" },
  failedBefore: { icon: CircleX, word: "Failed last time", className: "text-destructive/60" },
  fromLastRun: { icon: History, word: "From last run", className: "text-muted-foreground" },
};

export function StatusIcon({ state }: { state: RowState }) {
  const look = state === "running" ? null : STATUS_ICONS[state];
  const word = look?.word ?? RUNNING_WORD;
  return (
    <Tooltip>
      <TooltipTrigger
        render={<span className="flex size-4 shrink-0 items-center justify-center" />}
      >
        {look ? (
          <look.icon aria-label={word} className={cn("size-4", look.className)} />
        ) : (
          <Spinner aria-label={word} />
        )}
      </TooltipTrigger>
      <TooltipContent>{word}</TooltipContent>
    </Tooltip>
  );
}
