import { Input } from "@/renderer/components/ui/input";

export function InlineEdit({
  value,
  className,
  onCommit,
  onDone,
  onPointerDown,
}: {
  value: string;
  className?: string;
  onCommit: (text: string) => void;
  onDone: () => void;
  onPointerDown?: (e: React.PointerEvent<HTMLInputElement>) => void;
}) {
  return (
    <Input
      autoFocus
      defaultValue={value}
      className={className}
      onPointerDown={onPointerDown}
      onKeyDown={(e) => {
        e.stopPropagation();
        if (e.key === "Enter" || e.key === "Escape") e.currentTarget.blur();
      }}
      onBlur={(e) => {
        const text = e.currentTarget.value.trim();
        onDone();
        if (text && text !== value) onCommit(text);
      }}
    />
  );
}
