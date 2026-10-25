import type { ComponentProps } from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export function ListRow({
  active = false,
  className,
  ...props
}: Omit<ComponentProps<typeof Button>, "variant" | "size"> & { active?: boolean }) {
  return (
    <Button
      variant="ghost"
      size="sm"
      className={cn(
        "h-auto w-full justify-start gap-2 py-1.5 text-left",
        active && "bg-muted",
        className,
      )}
      {...props}
    />
  );
}
