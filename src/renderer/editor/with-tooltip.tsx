import type { LucideIcon } from "lucide-react";
import type { ComponentProps, ReactElement, ReactNode } from "react";
import { Button } from "@/renderer/components/ui/button";
import { Kbd } from "@/renderer/components/ui/kbd";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";

export function WithTooltip({
  trigger,
  tip,
  tipClassName,
  children,
}: {
  trigger: ReactElement;
  tip: ReactNode;
  tipClassName?: string;
  children: ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger render={trigger}>{children}</TooltipTrigger>
      <TooltipContent className={tipClassName}>{tip}</TooltipContent>
    </Tooltip>
  );
}

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
    <WithTooltip
      trigger={
        <Button
          variant="ghost"
          size={size}
          aria-label={label}
          disabled={disabled}
          onClick={onClick}
        />
      }
      tip={
        <>
          {label}
          {shortcut ? (
            <>
              {" "}
              <Kbd>{shortcut}</Kbd>
            </>
          ) : null}
        </>
      }
    >
      <Icon />
    </WithTooltip>
  );
}
