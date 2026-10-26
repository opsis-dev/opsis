import { Check, ChevronsUpDown, Settings2 } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import type { FontChoice, FontChoices, FontFamilyDef } from "@/generated/bindings";
import { reportError } from "@/lib/toast";
import { useBusy } from "@/lib/use-busy";
import { useWorkflowStore, workflowEdits } from "../store/workflow";
import { FontSample } from "./font-sample";

function SelectedMark({ selected }: { selected: boolean }) {
  return selected ? <Check data-icon="inline-end" className="ml-auto" /> : null;
}

function FontCommandItem({
  choice,
  onPick,
}: {
  choice: FontChoice;
  onPick: (choice: FontChoice) => void;
}) {
  return (
    <CommandItem value={choice.def.family} onSelect={() => onPick(choice)}>
      <FontSample choice={choice}>{choice.def.family}</FontSample>
      <SelectedMark selected={choice.selected} />
    </CommandItem>
  );
}

const NO_CHOICES: FontChoices = { project: [], builtIn: [], library: [] };

const NO_FAMILIES: FontFamilyDef[] = [];

const fetchChoices = (value: string, project: FontFamilyDef[], show: (c: FontChoices) => void) =>
  window.opsis
    .fontChoices(value, project)
    .then(show, (e: unknown) => reportError("Couldn't list the fonts", e));

function useFontChoices(value: string): [FontChoices, () => void] {
  const project = useWorkflowStore((s) => s.workflow?.settings.fonts.families) ?? NO_FAMILIES;
  const [choices, setChoices] = useState(NO_CHOICES);
  useEffect(() => {
    let current = true;
    void fetchChoices(value, project, (found) => current && setChoices(found));
    return () => {
      current = false;
    };
  }, [value, project]);
  return [choices, () => void fetchChoices(value, project, setChoices)];
}

async function copyIntoProject(def: FontFamilyDef): Promise<FontFamilyDef> {
  const projectDir = useWorkflowStore.getState().projectDir;
  if (!projectDir) throw new Error("Open a project to use this font.");
  const copied = await window.opsis.useFontInProject(projectDir, def.id);
  workflowEdits.addProjectFont(copied);
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
  const [{ project, builtIn, library, current }, refresh] = useFontChoices(value);

  const toggle = (next: boolean) => {
    setOpen(next);
    if (next) refresh();
  };

  const choose = (choice: FontChoice | null) => {
    setOpen(false);
    const def = choice?.def;
    if (def && library.includes(choice))
      void run(def.id, async () => onChange((await copyIntoProject(def)).family));
    else onChange(def?.family ?? "");
  };

  const items = (choices: FontChoice[]) =>
    choices.map((choice) => (
      <FontCommandItem key={choice.preview} choice={choice} onPick={choose} />
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
              <FontSample choice={current}>{value}</FontSample>
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
            {library.length ? (
              <CommandGroup heading="Your library">{items(library)}</CommandGroup>
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
