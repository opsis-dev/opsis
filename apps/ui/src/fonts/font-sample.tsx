import { fontStack } from "@opsis/video/common/fonts";
import type { ReactNode } from "react";
import type { FontFamilyDef } from "@/generated/bindings";
import { cn } from "@/lib/utils";
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
      style={shown ? { fontFamily: fontStack(shown) } : undefined}
    >
      {children}
    </span>
  );
}
