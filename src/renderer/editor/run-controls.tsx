import { Square } from "lucide-react";
import { Button } from "@/renderer/components/ui/button";
import { TextShimmer } from "@/renderer/components/ui/text-shimmer";

export function RunningLabel({ className }: { className?: string }) {
  return (
    <TextShimmer as="span" className={className} duration={1.6}>
      Running…
    </TextShimmer>
  );
}

export function StopRunButton({ size }: { size: "xs" | "sm" }) {
  return (
    <Button size={size} variant="outline" onClick={() => void window.demoflow.cancelRun()}>
      <Square data-icon="inline-start" />
      Stop
    </Button>
  );
}
