import { useRef, useState } from "react";
import { Input } from "@/renderer/components/ui/input";
import { nonBlankString } from "@/shared/guards";

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
        const text = nonBlankString(e.currentTarget.value);
        const keep = cancelled.current;
        cancelled.current = false;
        onDone?.();
        if (!keep && text && text !== value) onCommit(text);
        e.currentTarget.value = value;
      }}
    />
  );
}

export function useRenaming(value: string, onRename: (text: string) => void, className: string) {
  const [renaming, setRenaming] = useState(false);
  const input = renaming ? (
    <InlineEdit
      value={value}
      className={className}
      onPointerDown={(e) => e.stopPropagation()}
      onDone={() => setRenaming(false)}
      onCommit={onRename}
    />
  ) : null;
  return { input, startRenaming: () => setRenaming(true) };
}
