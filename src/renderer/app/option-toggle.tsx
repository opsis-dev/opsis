import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { whenFirstOneOf } from "@/renderer/lib/one-of";

export function OptionToggle<T extends string>({
  options,
  labels,
  value,
  onChange,
}: {
  options: readonly T[];
  labels: Record<T, string>;
  value: T;
  onChange: (value: T) => void;
}) {
  return (
    <ToggleGroup
      variant="outline"
      size="sm"
      value={[value]}
      onValueChange={whenFirstOneOf(options, onChange)}
    >
      {options.map((option) => (
        <ToggleGroupItem key={option} value={option}>
          {labels[option]}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  );
}
