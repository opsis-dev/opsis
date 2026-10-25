import { nonBlankString } from "@opsis/core/guards";
import { FolderOpen } from "lucide-react";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/components/ui/input-group";

export function FolderField({
  id,
  value,
  title,
  onChange,
  onPicked,
  onCommit,
}: {
  id: string;
  value: string;
  title: string;
  onChange: (value: string) => void;
  onPicked: (dir: string) => void;
  onCommit?: () => void;
}) {
  async function browse() {
    const picked = await window.opsis.chooseFolder({
      title,
      defaultPath: nonBlankString(value),
    });
    if (picked) onPicked(picked);
  }

  return (
    <InputGroup>
      <InputGroupInput
        id={id}
        value={value}
        spellCheck={false}
        onChange={(e) => onChange(e.target.value)}
        onBlur={onCommit}
        onKeyDown={(e) => e.key === "Enter" && onCommit?.()}
      />
      <InputGroupAddon align="inline-end">
        <InputGroupButton size="xs" onClick={() => void browse()}>
          <FolderOpen data-icon="inline-start" />
          Browse
        </InputGroupButton>
      </InputGroupAddon>
    </InputGroup>
  );
}
