import type { ErrorComponentProps } from "@tanstack/react-router";
import { ChevronRight, RotateCcw, TriangleAlert } from "lucide-react";
import { useEffect } from "react";
import { Button } from "@/renderer/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/renderer/components/ui/collapsible";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/renderer/components/ui/empty";
import { AppBar } from "./app-bar";

export function ErrorScreen({ error, reset }: ErrorComponentProps) {
  const message = error instanceof Error ? error.message : String(error);

  useEffect(() => console.error(error), [error]);

  return (
    <div className="bg-background flex h-screen flex-col">
      <AppBar />
      <Empty className="flex-1">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <TriangleAlert />
          </EmptyMedia>
          <EmptyTitle>Something went wrong</EmptyTitle>
          <EmptyDescription>
            Your project is saved. Try again to pick up where you were, or reload the window.
          </EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
          <div className="flex gap-2">
            <Button size="sm" onClick={reset}>
              Try again
            </Button>
            <Button size="sm" variant="outline" onClick={() => window.location.reload()}>
              <RotateCcw data-icon="inline-start" />
              Reload window
            </Button>
          </div>
          <Collapsible className="w-full max-w-lg">
            <CollapsibleTrigger
              render={
                <Button size="sm" variant="ghost" className="text-muted-foreground group/details" />
              }
            >
              <ChevronRight
                data-icon="inline-start"
                className="group-data-[panel-open]/details:rotate-90"
              />
              Details
            </CollapsibleTrigger>
            <CollapsibleContent>
              <pre className="bg-muted text-muted-foreground mt-2 rounded-md p-3 text-left font-mono text-xs break-words whitespace-pre-wrap">
                {message}
              </pre>
            </CollapsibleContent>
          </Collapsible>
        </EmptyContent>
      </Empty>
    </div>
  );
}
