import { Check, ChevronsUpDown, Settings2 } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { BUNDLED_FONTS, familyKey, sameFamily, type FontFamilyDef } from "@/shared/fonts";
import { Button } from "@/renderer/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@/renderer/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@/renderer/components/ui/popover";
import { useWorkflowStore } from "../store/workflow";
import { useFontLibrary, usePreviewFamily } from "./library";
import { errorMessage } from "@/shared/errors";

const NO_FAMILIES: FontFamilyDef[] = [];

function FontName({ def, name }: { def?: FontFamilyDef; name: string }) {
  const family = usePreviewFamily(def);
  return (
    <span
      className="truncate"
      style={family ? { fontFamily: `"${family}", sans-serif` } : undefined}
    >
      {name}
    </span>
  );
}

export function FontPicker({
  id,
  value,
  onChange,
  emptyLabel,
}: {
  id?: string;
  value: string;
  onChange: (family: string) => void;
  emptyLabel?: string;
}) {
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const project = useWorkflowStore((s) => s.workflow?.settings.fonts.families) ?? NO_FAMILIES;
  const library = useFontLibrary((s) => s.library);

  const toggle = (next: boolean) => {
    setOpen(next);
    if (next) useFontLibrary.getState().ensureLoaded();
  };

  const taken = new Set([...project, ...BUNDLED_FONTS].map((f) => familyKey(f.family)));
  const extra = library.filter((f) => !taken.has(familyKey(f.family)));
  const builtIn = BUNDLED_FONTS.filter((d) => !project.some((p) => sameFamily(p.family, d.family)));
  const current = [...project, ...BUNDLED_FONTS, ...library].find((f) =>
    sameFamily(f.family, value),
  );

  const pick = async (family: string, fromLibrary?: FontFamilyDef) => {
    setOpen(false);
    if (!fromLibrary) return onChange(family);
    setBusy(true);
    try {
      const projectDir = useWorkflowStore.getState().projectDir;
      if (!projectDir) throw new Error("Open a project to use this font.");
      const def = await window.demoflow.useFontInProject(projectDir, fromLibrary.id);
      useWorkflowStore.getState().addProjectFont(def);
      onChange(def.family);
    } catch (error) {
      toast.error(errorMessage(error));
    } finally {
      setBusy(false);
    }
  };

  const item = (def: FontFamilyDef, fromLibrary?: boolean) => (
    <CommandItem
      key={`${def.source}-${def.id}`}
      value={def.family}
      onSelect={() => void pick(def.family, fromLibrary ? def : undefined)}
    >
      <FontName def={def} name={def.family} />
      {sameFamily(value, def.family) ? <Check data-icon="inline-end" className="ml-auto" /> : null}
    </CommandItem>
  );

  return (
    <Popover open={open} onOpenChange={toggle}>
      <PopoverTrigger
        render={
          <Button
            id={id}
            variant="outline"
            className="w-full justify-between font-normal"
            disabled={busy}
          >
            {value ? (
              <FontName def={current} name={value} />
            ) : (
              <span className="text-muted-foreground truncate">
                {emptyLabel ?? "Choose a font"}
              </span>
            )}
            <ChevronsUpDown data-icon="inline-end" />
          </Button>
        }
      />
      <PopoverContent className="w-72 p-0" align="start">
        <Command>
          <CommandInput placeholder="Search fonts…" />
          <CommandList>
            <CommandEmpty>No font found.</CommandEmpty>
            {emptyLabel ? (
              <CommandGroup>
                <CommandItem value={emptyLabel} onSelect={() => void pick("")}>
                  {emptyLabel}
                  {!value ? <Check data-icon="inline-end" className="ml-auto" /> : null}
                </CommandItem>
              </CommandGroup>
            ) : null}
            {project.length ? (
              <CommandGroup heading="In this project">{project.map((d) => item(d))}</CommandGroup>
            ) : null}
            <CommandGroup heading="Built in">{builtIn.map((d) => item(d))}</CommandGroup>
            {extra.length ? (
              <CommandGroup heading="Your library">{extra.map((d) => item(d, true))}</CommandGroup>
            ) : null}
            <CommandSeparator />
            <CommandGroup>
              <CommandItem
                value="manage fonts"
                onSelect={() => {
                  setOpen(false);
                  void window.demoflow.openSettings("fonts");
                }}
              >
                <Settings2 />
                Add or manage fonts…
              </CommandItem>
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
