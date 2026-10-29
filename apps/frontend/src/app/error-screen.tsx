import type { ErrorComponentProps } from "@tanstack/react-router";
import { ChevronRight, RotateCcw, TriangleAlert } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { errorMessage } from "@/lib/errors";
import { AppBar } from "./app-bar";
import { EmptyPanel } from "./empty-panel";

export function ErrorScreen({ error, reset }: ErrorComponentProps) {
  const message = errorMessage(error);

  return (
    <div className="bg-background flex h-screen flex-col">
      <AppBar />
      <EmptyPanel
        icon={TriangleAlert}
        title="Something went wrong"
        description="Your project is saved. Try again to pick up where you were, or reload the window."
        className="flex-1"
      >
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
      </EmptyPanel>
    </div>
  );
}
