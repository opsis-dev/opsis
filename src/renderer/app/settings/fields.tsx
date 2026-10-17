import { useCallback, useEffect, useRef, useState, type ComponentProps } from "react";
import { Field, FieldDescription, FieldLabel } from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { Switch } from "@/renderer/components/ui/switch";

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
  const [draft, setDraft] = useState<string | null>(null);
  const pending = useRef<{ text: string; commit: (text: string) => void } | null>(null);
  const flush = useCallback(() => {
    const unsaved = pending.current;
    pending.current = null;
    unsaved?.commit(unsaved.text);
  }, []);
  useEffect(() => {
    window.addEventListener("pagehide", flush);
    return () => {
      window.removeEventListener("pagehide", flush);
      flush();
    };
  }, [flush]);
  const commit = () => {
    if (draft === null) return;
    setDraft(null);
    pending.current = null;
    if (draft !== value) onCommit(draft);
  };
  return (
    <Input
      {...props}
      value={draft ?? value}
      onChange={(e) => {
        setDraft(e.target.value);
        pending.current = { text: e.target.value, commit: onCommit };
      }}
      onBlur={commit}
      onKeyDown={(e) => e.key === "Enter" && commit()}
    />
  );
}
