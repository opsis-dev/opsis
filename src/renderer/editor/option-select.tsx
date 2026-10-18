import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";

export function OptionSelect<T extends string>({
  label,
  value,
  options,
  optionLabel,
  className,
  onChange,
}: {
  label: string;
  value: T;
  options: readonly T[];
  optionLabel: (value: T) => string;
  className: string;
  onChange: (value: T) => void;
}) {
  return (
    <Select
      value={value}
      onValueChange={(v) => {
        if (v) onChange(v);
      }}
    >
      <SelectTrigger size="sm" className={className} aria-label={label}>
        <SelectValue>{(v: T) => optionLabel(v)}</SelectValue>
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          {options.map((option) => (
            <SelectItem key={option} value={option}>
              {optionLabel(option)}
            </SelectItem>
          ))}
        </SelectGroup>
      </SelectContent>
    </Select>
  );
}
