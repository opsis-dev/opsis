import { Check, ChevronsUpDown, Settings2 } from "lucide-react";
import { useState } from "react";
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
import { useBusy } from "@/renderer/lib/use-busy";
import { familyKey, familyKeysOf, familyKeysWithBundled, sameFamily } from "@/shared/fonts";
import { BUNDLED_FONTS } from "@/shared/fonts/bundled.generated";
import type { FontFamilyDef } from "@/shared/fonts/types";
import { useWorkflowStore } from "../store/workflow";
import { FontSample } from "./font-sample";
import { useFontLibrary } from "./library";

const NO_FAMILIES: FontFamilyDef[] = [];

function SelectedMark({ selected }: { selected: boolean }) {
  return selected ? <Check data-icon="inline-end" className="ml-auto" /> : null;
}

function FontCommandItem({
  def,
  selected,
  onPick,
}: {
  def: FontFamilyDef;
  selected: boolean;
  onPick: (def: FontFamilyDef) => void;
}) {
  return (
    <CommandItem value={def.family} onSelect={() => onPick(def)}>
      <FontSample def={def}>{def.family}</FontSample>
      <SelectedMark selected={selected} />
    </CommandItem>
  );
}

const without = (defs: FontFamilyDef[], keys: ReadonlySet<string>): FontFamilyDef[] =>
  defs.filter((d) => !keys.has(familyKey(d.family)));

function fontGroups(project: FontFamilyDef[], library: FontFamilyDef[], value: string) {
  return {
    builtIn: without(BUNDLED_FONTS, familyKeysOf(project)),
    extra: without(library, familyKeysWithBundled(project)),
    current: [...project, ...BUNDLED_FONTS, ...library].find((f) => sameFamily(f.family, value)),
  };
}

async function copyIntoProject(def: FontFamilyDef): Promise<FontFamilyDef> {
  const projectDir = useWorkflowStore.getState().projectDir;
  if (!projectDir) throw new Error("Open a project to use this font.");
  const copied = await window.opsis.useFontInProject(projectDir, def.id);
  useWorkflowStore.getState().addProjectFont(copied);
  return copied;
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
  const { isBusy, run } = useBusy();
  const project = useWorkflowStore((s) => s.workflow?.settings.fonts.families) ?? NO_FAMILIES;
  const library = useFontLibrary((s) => s.library);

  const toggle = (next: boolean) => {
    setOpen(next);
    if (next) useFontLibrary.getState().ensureLoaded();
  };

  const { builtIn, extra, current } = fontGroups(project, library, value);

  const choose = (def: FontFamilyDef | null) => {
    setOpen(false);
    if (def && extra.includes(def))
      void run(def.id, async () => onChange((await copyIntoProject(def)).family));
    else onChange(def?.family ?? "");
  };

  const items = (defs: FontFamilyDef[]) =>
    defs.map((def) => (
      <FontCommandItem
        key={`${def.source}-${def.id}`}
        def={def}
        selected={sameFamily(value, def.family)}
        onPick={choose}
      />
    ));

  return (
    <Popover open={open} onOpenChange={toggle}>
      <PopoverTrigger
        render={
          <Button
            id={id}
            variant="outline"
            className="w-full justify-between font-normal"
            disabled={isBusy}
          >
            {value ? (
              <FontSample def={current}>{value}</FontSample>
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
                <CommandItem value={emptyLabel} onSelect={() => choose(null)}>
                  {emptyLabel}
                  <SelectedMark selected={!value} />
                </CommandItem>
              </CommandGroup>
            ) : null}
            {project.length ? (
              <CommandGroup heading="In this project">{items(project)}</CommandGroup>
            ) : null}
            <CommandGroup heading="Built in">{items(builtIn)}</CommandGroup>
            {extra.length ? (
              <CommandGroup heading="Your library">{items(extra)}</CommandGroup>
            ) : null}
            <CommandSeparator />
            <CommandGroup>
              <CommandItem
                value="manage fonts"
                onSelect={() => {
                  setOpen(false);
                  void window.opsis.openSettings("fonts");
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
