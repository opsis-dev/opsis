import { Ellipsis } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { busSettings, stripSettings, type StripSettings } from "@/editor/mixer/strip-settings";
import type { BusPatch, Strip, StripPatch } from "@/generated/bindings";
import { mixerBook } from "@/store/catalog";
import { useMixerState } from "./mixer-state";

type Apply<T> = (patch: T) => void;

const busPart = (settings: Partial<StripSettings>): BusPatch => busSettings(settings);

function SettingsMenu({
  label,
  className,
  children,
}: {
  label: string;
  className: string;
  children: React.ReactNode;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger render={<Button variant="ghost" size="icon-xs" aria-label={label} />}>
        <Ellipsis />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className={className}>
        {children}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function PresetItems({
  disabled,
  onApply,
}: {
  disabled?: boolean;
  onApply: (settings: StripPatch) => void;
}) {
  return mixerBook().presets.map((p) => (
    <DropdownMenuItem key={p.id} disabled={disabled} onClick={() => onApply(p.strip)}>
      {p.label}
    </DropdownMenuItem>
  ));
}

function PasteItem({
  disabled,
  onPaste,
  children,
}: {
  disabled?: boolean;
  onPaste: (settings: StripSettings) => void;
  children: React.ReactNode;
}) {
  const clipboard = useMixerState((s) => s.clipboard);
  return (
    <DropdownMenuItem
      disabled={disabled || !clipboard}
      onClick={() => clipboard && onPaste(clipboard)}
    >
      {children}
    </DropdownMenuItem>
  );
}

export function StripMenu({
  strip,
  count,
  onApply,
  onReset,
}: {
  strip: Strip;
  count: number;
  onApply: Apply<StripPatch>;
  onReset: () => void;
}) {
  const several = count > 1;
  const paste = onApply;
  return (
    <SettingsMenu label="Strip settings" className="w-52">
      <DropdownMenuGroup>
        <DropdownMenuLabel>
          {several ? `Presets for ${count} selected strips` : "Presets"}
        </DropdownMenuLabel>
        <PresetItems onApply={paste} />
      </DropdownMenuGroup>
      <DropdownMenuSeparator />
      <DropdownMenuGroup>
        <DropdownMenuItem onClick={() => useMixerState.getState().copy(stripSettings(strip))}>
          Copy settings
        </DropdownMenuItem>
        <PasteItem onPaste={paste}>
          {several ? `Paste onto ${count} strips` : "Paste settings"}
        </PasteItem>
        <DropdownMenuItem onClick={onReset}>
          {several ? `Reset ${count} strips` : "Reset"}
        </DropdownMenuItem>
      </DropdownMenuGroup>
    </SettingsMenu>
  );
}

export function BusMenu({
  sounds,
  onBus,
  onBusReset,
  onSounds,
  onSoundsReset,
}: {
  sounds: number;
  onBus: Apply<BusPatch>;
  onBusReset: () => void;
  onSounds: Apply<StripPatch>;
  onSoundsReset: () => void;
}) {
  const toBus = (settings: Partial<StripSettings>) => onBus(busPart(settings));
  const toSounds = onSounds;
  return (
    <SettingsMenu label="Bus settings" className="w-56">
      <DropdownMenuGroup>
        <DropdownMenuLabel>This bus</DropdownMenuLabel>
        <PresetItems onApply={toBus} />
        <PasteItem onPaste={toBus}>Paste settings</PasteItem>
        <DropdownMenuItem onClick={onBusReset}>Reset</DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />
      <DropdownMenuGroup>
        <DropdownMenuLabel>
          {sounds === 1 ? "The sound on this bus" : `All ${sounds} sounds on this bus`}
        </DropdownMenuLabel>
        <PresetItems disabled={!sounds} onApply={toSounds} />
        <PasteItem disabled={!sounds} onPaste={toSounds}>
          Paste settings
        </PasteItem>
        <DropdownMenuItem disabled={!sounds} onClick={onSoundsReset}>
          Reset
        </DropdownMenuItem>
      </DropdownMenuGroup>
    </SettingsMenu>
  );
}
