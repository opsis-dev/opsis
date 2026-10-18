import { useRef } from "react";
import { Input } from "@/renderer/components/ui/input";

export function InlineEdit({
  value,
  className,
  autoFocus = true,
  label,
  onCommit,
  onDone,
  onPointerDown,
}: {
  value: string;
  className?: string;
  autoFocus?: boolean;
  label?: string;
  onCommit: (text: string) => void;
  onDone?: () => void;
  onPointerDown?: (e: React.PointerEvent<HTMLInputElement>) => void;
}) {
  const cancelled = useRef(false);
  return (
    <Input
      autoFocus={autoFocus}
      aria-label={label}
      defaultValue={value}
      className={className}
      onPointerDown={onPointerDown}
      onKeyDown={(e) => {
        e.stopPropagation();
        if (e.key === "Escape") {
          cancelled.current = true;
          e.currentTarget.value = value;
        }
        if (e.key === "Enter" || e.key === "Escape") e.currentTarget.blur();
      }}
      onBlur={(e) => {
        const text = e.currentTarget.value.trim();
        const keep = cancelled.current;
        cancelled.current = false;
        onDone?.();
        if (!keep && text && text !== value) onCommit(text);
        e.currentTarget.value = value;
      }}
    />
  );
}
