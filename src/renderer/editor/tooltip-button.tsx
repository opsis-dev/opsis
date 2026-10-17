import type { LucideIcon } from "lucide-react";
import type { ComponentProps } from "react";
import { Button } from "@/renderer/components/ui/button";
import { Kbd } from "@/renderer/components/ui/kbd";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";

export function TooltipButton({
  label,
  icon: Icon,
  onClick,
  shortcut,
  disabled,
  size = "icon-sm",
}: {
  label: string;
  icon: LucideIcon;
  onClick: () => void;
  shortcut?: string | null;
  disabled?: boolean;
  size?: ComponentProps<typeof Button>["size"];
}) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <Button
            variant="ghost"
            size={size}
            aria-label={label}
            disabled={disabled}
            onClick={onClick}
          />
        }
      >
        <Icon />
      </TooltipTrigger>
      <TooltipContent>
        {label}
        {shortcut ? (
          <>
            {" "}
            <Kbd>{shortcut}</Kbd>
          </>
        ) : null}
      </TooltipContent>
    </Tooltip>
  );
}
