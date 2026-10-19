import { Copy, Minus, Square, X } from "lucide-react";
import type { ReactNode } from "react";
import { isMac } from "@/renderer/lib/platform";
import { cn } from "@/renderer/lib/utils";
import type { WindowRole } from "@/shared/menu";
import { useWindowState } from "../store/window-state";

function WindowButton({
  role,
  label,
  className = "hover:bg-accent",
  children,
}: {
  role: WindowRole;
  label: string;
  className?: string;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      className={cn(
        "text-muted-foreground hover:text-foreground grid h-full w-12 place-items-center transition-colors",
        className,
      )}
      onClick={() => void window.opsis.windowRole(role)}
    >
      {children}
    </button>
  );
}

export function WindowControls() {
  const maximized = useWindowState((s) => s.maximized);
  return (
    <div className="flex h-full shrink-0 items-stretch">
      <WindowButton role="minimize" label="Minimize">
        <Minus className="size-4" strokeWidth={1.5} />
      </WindowButton>
      <WindowButton role="toggleMaximize" label={maximized ? "Restore" : "Maximize"}>
        {maximized ? (
          <Copy className="size-3.5 -scale-x-100" strokeWidth={1.5} />
        ) : (
          <Square className="size-3.5" strokeWidth={1.5} />
        )}
      </WindowButton>
      <WindowButton role="close" label="Close" className="hover:bg-destructive hover:text-white">
        <X className="size-4" strokeWidth={1.5} />
      </WindowButton>
    </div>
  );
}

export function WindowBorder() {
  const { maximized, fullscreen, focused } = useWindowState();
  if (isMac || maximized || fullscreen) return null;
  return (
    <div
      aria-hidden
      className={cn(
        "pointer-events-none fixed inset-0 z-50 border",
        focused ? "border-white/12" : "border-white/7",
      )}
    />
  );
}
