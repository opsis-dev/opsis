import type { ReactNode } from "react";
import type { FontChoice } from "@/generated/bindings";
import { cn } from "@/lib/utils";
import { usePreviewFont } from "./library";

export function FontSample({
  choice,
  stack,
  className,
  children,
}: {
  choice?: FontChoice;
  stack?: string;
  className?: string;
  children: ReactNode;
}) {
  usePreviewFont(choice);
  const shown = choice?.stack ?? stack;
  return (
    <span className={cn("truncate", className)} style={shown ? { fontFamily: shown } : undefined}>
      {children}
    </span>
  );
}
