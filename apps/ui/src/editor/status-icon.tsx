import { Circle, CircleCheck, CircleSlash, CircleX, History, Radio } from "lucide-react";
import type { ComponentProps, ComponentType } from "react";
import { Spinner } from "@/components/ui/spinner";
import type { NodeRunStatus } from "@/generated/bindings";
import { cn } from "@/lib/utils";
import { WithTooltip } from "./with-tooltip";

export type RowState = NodeRunStatus | "failedBefore" | "fromLastRun";

type StatusLook = {
  icon: ComponentType<ComponentProps<"svg">>;
  word: string;
  short?: string;
  className?: string;
  badge?: "default" | "secondary" | "destructive";
};

export const STATUS_LOOKS: Record<RowState | "live", StatusLook> = {
  idle: { icon: Circle, word: "Not run", className: "text-muted-foreground" },
  queued: { icon: Circle, word: "Waiting", className: "text-muted-foreground" },
  running: { icon: Spinner, word: "Running" },
  done: { icon: CircleCheck, word: "Done", className: "text-success", badge: "secondary" },
  cached: { icon: CircleCheck, word: "Up to date", className: "text-muted-foreground" },
  skipped: { icon: CircleSlash, word: "Skipped", className: "text-muted-foreground" },
  error: { icon: CircleX, word: "Failed", className: "text-destructive", badge: "destructive" },
  failedBefore: { icon: CircleX, word: "Failed last time", className: "text-destructive/60" },
  fromLastRun: { icon: History, word: "From last run", className: "text-muted-foreground" },
  live: { icon: Radio, word: "Recording", short: "Live", badge: "default" },
};

export function StatusIcon({ state }: { state: RowState }) {
  const { icon: Icon, word, className } = STATUS_LOOKS[state];
  return (
    <WithTooltip
      trigger={<span className="flex size-4 shrink-0 items-center justify-center" />}
      tip={word}
    >
      <Icon aria-label={word} className={cn("size-4", className)} />
    </WithTooltip>
  );
}
