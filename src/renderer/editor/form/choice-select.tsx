import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";

export type Choice = { value: string; label: string; disabled?: boolean; hint?: string };

const NONE_VALUE = "__none";

export function ChoiceSelect({
  id,
  groups,
  value,
  onChange,
  noneLabel,
  placeholder,
  size,
  className = "w-full",
  "aria-label": ariaLabel,
}: {
  id?: string;
  groups: Choice[][];
  value: string | null;
  onChange: (value: string) => void;
  noneLabel?: string;
  placeholder?: string;
  size?: "sm" | "default";
  className?: string;
  "aria-label"?: string;
}) {
  const [first = [], ...rest] = groups;
  const lists = noneLabel ? [[{ value: NONE_VALUE, label: noneLabel }, ...first], ...rest] : groups;
  const selected = noneLabel && !value ? NONE_VALUE : value;
  return (
    <Select
      items={lists.flat()}
      value={selected}
      onValueChange={(v) => {
        if (v != null) onChange(v === NONE_VALUE ? "" : String(v));
      }}
    >
      <SelectTrigger id={id} size={size} className={className} aria-label={ariaLabel}>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent>
        {lists.map((list, i) => (
          <SelectGroup key={i}>
            {list.map((c) => (
              <SelectItem key={c.value} value={c.value} disabled={c.disabled}>
                {c.label}
                {c.hint ? (
                  <span className="text-muted-foreground ml-1 text-xs">{c.hint}</span>
                ) : null}
              </SelectItem>
            ))}
          </SelectGroup>
        ))}
      </SelectContent>
    </Select>
  );
}
