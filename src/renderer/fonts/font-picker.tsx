import { Check, ChevronsUpDown, Settings2 } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { BUNDLED_FONTS, type FontFamilyDef } from "@/shared/fonts";
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
import { registerPreview, useFontLibrary } from "./library";

function FontName({ def, name }: { def?: FontFamilyDef; name: string }) {
  const family = def ? registerPreview(def) : undefined;
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
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const fonts = useWorkflowStore((s) => s.workflow?.settings.fonts);
  const { library, loaded, refresh } = useFontLibrary();
  useEffect(() => {
    if (open && !loaded) void refresh();
  }, [open, loaded, refresh]);

  const project = fonts?.families ?? [];
  const taken = new Set([...project, ...BUNDLED_FONTS].map((f) => f.family.toLowerCase()));
  const extra = library.filter((f) => !taken.has(f.family.toLowerCase()));
  const current = [...project, ...BUNDLED_FONTS, ...library].find(
    (f) => f.family.toLowerCase() === value.toLowerCase(),
  );

  const pick = async (family: string, fromLibrary?: FontFamilyDef) => {
    setOpen(false);
    if (!fromLibrary) return onChange(family);
    if (!projectDir || !fonts) return;
    setBusy(true);
    try {
      const def = await window.demoflow.useFontInProject(projectDir, fromLibrary.id);
      const { setSettings, workflow } = useWorkflowStore.getState();
      const latest = workflow?.settings.fonts ?? fonts;
      setSettings({
        fonts: {
          ...latest,
          families: [
            ...latest.families.filter((f) => f.family.toLowerCase() !== def.family.toLowerCase()),
            def,
          ],
        },
      });
      onChange(def.family);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
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
      {value.toLowerCase() === def.family.toLowerCase() ? (
        <Check data-icon="inline-end" className="ml-auto" />
      ) : null}
    </CommandItem>
  );

  return (
    <Popover open={open} onOpenChange={setOpen}>
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
            <CommandGroup heading="Built in">
              {BUNDLED_FONTS.filter(
                (d) => !project.some((p) => p.family.toLowerCase() === d.family.toLowerCase()),
              ).map((d) => item(d))}
            </CommandGroup>
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
