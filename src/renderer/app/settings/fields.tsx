import type { ComponentProps } from "react";
import { Field, FieldDescription, FieldLabel } from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { Switch } from "@/renderer/components/ui/switch";
import { useCommitDraft } from "@/renderer/lib/use-commit-draft";

export function SwitchField({
  id,
  label,
  description,
  checked,
  onChange,
}: {
  id: string;
  label: string;
  description?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <Field orientation="horizontal" className="justify-between">
      <div className="flex flex-col gap-1">
        <FieldLabel htmlFor={id} className="font-normal">
          {label}
        </FieldLabel>
        {description ? <FieldDescription>{description}</FieldDescription> : null}
      </div>
      <Switch id={id} checked={checked} onCheckedChange={onChange} />
    </Field>
  );
}

export function CommitInput({
  value,
  onCommit,
  ...props
}: Omit<ComponentProps<typeof Input>, "value" | "onChange" | "onBlur" | "onKeyDown"> & {
  value: string;
  onCommit: (text: string) => void;
}) {
  const draft = useCommitDraft(value, onCommit);
  return (
    <Input
      {...props}
      value={draft.text}
      onChange={(e) => draft.change(e.target.value)}
      onBlur={draft.commit}
      onKeyDown={(e) => e.key === "Enter" && draft.commit()}
    />
  );
}
