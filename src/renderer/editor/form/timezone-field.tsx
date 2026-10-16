import { Check, ChevronsUpDown } from "lucide-react";
import { useState } from "react";
import { Button } from "@/renderer/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/renderer/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@/renderer/components/ui/popover";
import { cn } from "@/renderer/lib/utils";
import { FieldShell } from "./field-shell";
import type { FieldProps } from "./types";

const ZONES = Intl.supportedValuesOf("timeZone");
const SAME_AS_COMPUTER = "Same as this computer";

const zoneName = (zone: string) => zone.replaceAll("_", " ");

export function TimezoneField({ label, value, onChange }: FieldProps<string>) {
  const [open, setOpen] = useState(false);
  const pick = (zone: string) => {
    onChange(zone);
    setOpen(false);
  };
  return (
    <FieldShell label={label}>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger
          render={
            <Button variant="outline" className="w-full justify-between font-normal">
              <span className={cn("truncate", !value && "text-muted-foreground")}>
                {value ? zoneName(value) : SAME_AS_COMPUTER}
              </span>
              <ChevronsUpDown data-icon="inline-end" />
            </Button>
          }
        />
        <PopoverContent className="w-72 p-0" align="start">
          <Command>
            <CommandInput placeholder="Search time zones…" />
            <CommandList>
              <CommandEmpty>No time zone found.</CommandEmpty>
              <CommandGroup>
                <CommandItem value="same as this computer" onSelect={() => pick("")}>
                  {SAME_AS_COMPUTER}
                  {!value ? <Check data-icon="inline-end" className="ml-auto" /> : null}
                </CommandItem>
                {ZONES.map((z) => (
                  <CommandItem key={z} value={zoneName(z)} onSelect={() => pick(z)}>
                    {zoneName(z)}
                    {value === z ? <Check data-icon="inline-end" className="ml-auto" /> : null}
                  </CommandItem>
                ))}
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    </FieldShell>
  );
}
