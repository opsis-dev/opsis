import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { whenFirstOneOf } from "@/lib/one-of";

export function OptionToggle<T extends string>({
  choices,
  value,
  onChange,
}: {
  choices: ReadonlyArray<readonly [T, string]>;
  value: T;
  onChange: (value: T) => void;
}) {
  return (
    <ToggleGroup
      variant="outline"
      size="sm"
      value={[value]}
      onValueChange={whenFirstOneOf(
        choices.map(([option]) => option),
        onChange,
      )}
    >
      {choices.map(([option, label]) => (
        <ToggleGroupItem key={option} value={option}>
          {label}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  );
}
