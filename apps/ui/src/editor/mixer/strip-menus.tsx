import { stripSettings } from "@opsis/core/mix/types";
import { STRIP_PRESETS, type Bus, type Strip, type StripSettings } from "@opsis/core/mix/types";
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
import { applyBusSettings, resetBus, resetStrip } from "@/mix/edits";
import { useMixerState } from "./mixer-state";

type Apply<T> = (fn: (value: T) => T) => void;

const applyToStrips =
  (onApply: Apply<Strip>) =>
  (settings: Partial<StripSettings>): void =>
    onApply((s) => ({ ...s, ...settings }));

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
  onApply: (settings: Partial<StripSettings>) => void;
}) {
  return STRIP_PRESETS.map((p) => (
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
}: {
  strip: Strip;
  count: number;
  onApply: Apply<Strip>;
}) {
  const several = count > 1;
  const paste = applyToStrips(onApply);
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
        <DropdownMenuItem onClick={() => onApply(resetStrip)}>
          {several ? `Reset ${count} strips` : "Reset"}
        </DropdownMenuItem>
      </DropdownMenuGroup>
    </SettingsMenu>
  );
}

export function BusMenu({
  sounds,
  onBus,
  onSounds,
}: {
  sounds: number;
  onBus: Apply<Bus>;
  onSounds: Apply<Strip>;
}) {
  const toBus = (settings: Partial<StripSettings>) => onBus((b) => applyBusSettings(b, settings));
  const toSounds = applyToStrips(onSounds);
  return (
    <SettingsMenu label="Bus settings" className="w-56">
      <DropdownMenuGroup>
        <DropdownMenuLabel>This bus</DropdownMenuLabel>
        <PresetItems onApply={toBus} />
        <PasteItem onPaste={toBus}>Paste settings</PasteItem>
        <DropdownMenuItem onClick={() => onBus(resetBus)}>Reset</DropdownMenuItem>
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
        <DropdownMenuItem disabled={!sounds} onClick={() => onSounds(resetStrip)}>
          Reset
        </DropdownMenuItem>
      </DropdownMenuGroup>
    </SettingsMenu>
  );
}
