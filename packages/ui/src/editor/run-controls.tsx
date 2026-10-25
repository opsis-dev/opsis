import { Square } from "lucide-react";
import { Button } from "@/components/ui/button";
import { TextShimmer } from "@/components/ui/text-shimmer";

export function RunningLabel({ className }: { className?: string }) {
  return (
    <TextShimmer as="span" className={className} duration={1.6}>
      Running…
    </TextShimmer>
  );
}

export function StopRunButton({ size }: { size: "xs" | "sm" }) {
  return (
    <Button size={size} variant="outline" onClick={() => void window.opsis.cancelRun()}>
      <Square data-icon="inline-start" />
      Stop
    </Button>
  );
}
