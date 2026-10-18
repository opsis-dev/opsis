import type { ReactNode } from "react";
import { cn } from "@/renderer/lib/utils";
import type { FontFamilyDef } from "@/shared/fonts/types";
import { usePreviewFamily } from "./library";

export function FontSample({
  def,
  family,
  className,
  children,
}: {
  def?: FontFamilyDef;
  family?: string;
  className?: string;
  children: ReactNode;
}) {
  const shown = usePreviewFamily(def) ?? family;
  return (
    <span
      className={cn("truncate", className)}
      style={shown ? { fontFamily: `"${shown}", sans-serif` } : undefined}
    >
      {children}
    </span>
  );
}
