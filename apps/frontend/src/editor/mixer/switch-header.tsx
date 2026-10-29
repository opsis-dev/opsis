import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";

export function SwitchHeader({
  title,
  checked,
  label,
  onChange,
}: {
  title: string;
  checked: boolean;
  label: string;
  onChange: (on: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <Label className="text-sm">{title}</Label>
      <Switch checked={checked} aria-label={label} onCheckedChange={onChange} />
    </div>
  );
}
