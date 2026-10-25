import { memo } from "react";
import { cn } from "@/lib/utils";
import type { LogLine } from "../store/run";
import { MONO_TEXT } from "./text-styles";

const LogRow = memo(function LogRow({ line, showTime }: { line: LogLine; showTime: boolean }) {
  return (
    <div
      className={cn(
        line.level === "error" && "text-destructive",
        line.level === "warn" && "text-muted-foreground",
      )}
    >
      {showTime ? (
        <span className="text-muted-foreground">{new Date(line.at).toLocaleTimeString()} </span>
      ) : null}
      {line.message}
    </div>
  );
});

export function LogView({
  lines,
  showTime = false,
  className,
}: {
  lines: LogLine[];
  showTime?: boolean;
  className?: string;
}) {
  return (
    <pre className={cn(MONO_TEXT, "leading-relaxed whitespace-pre-wrap", className)}>
      {lines.map((line) => (
        <LogRow key={line.id} line={line} showTime={showTime} />
      ))}
    </pre>
  );
}
