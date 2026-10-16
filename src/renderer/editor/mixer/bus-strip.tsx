import { ChevronLeft, ChevronRight, Trash2 } from "lucide-react";
import { useState } from "react";
import { removeBus, updateBus, updateChannels } from "@/shared/mix/edits";
import type { MixResult } from "@/shared/mix/result";
import { BUILT_IN_BUSES, type Bus, type Mix } from "@/shared/mix/types";
import { Button } from "@/renderer/components/ui/button";
import { Input } from "@/renderer/components/ui/input";
import { bindChange, setMix, share } from "./bindings";
import { DuckPopover } from "./dynamics-popovers";
import { levelAt } from "./meter";
import { BusMenu } from "./strip-menus";
import { HighpassKnob, StripLevel, StripShell, StripTone, type StripBinding } from "./strip-parts";

function BusName({ bus, onRename }: { bus: Bus; onRename: (name: string) => void }) {
  const [editing, setEditing] = useState(false);
  if (editing)
    return (
      <Input
        autoFocus
        defaultValue={bus.name}
        className="h-6 px-1 text-xs"
        onKeyDown={(e) => {
          e.stopPropagation();
          if (e.key === "Enter" || e.key === "Escape") e.currentTarget.blur();
        }}
        onBlur={(e) => {
          setEditing(false);
          const name = e.currentTarget.value.trim();
          if (name && name !== bus.name) onRename(name);
        }}
      />
    );
  return (
    <button
      type="button"
      className="flex-1 truncate px-1 text-left text-xs font-medium"
      title="Double-click to rename"
      onDoubleClick={() => setEditing(true)}
    >
      {bus.name}
    </button>
  );
}

function FoldButton({
  folded,
  members,
  onFold,
}: {
  folded: boolean;
  members: number;
  onFold: () => void;
}) {
  return (
    <button
      type="button"
      className="text-muted-foreground hover:text-foreground flex min-w-0 flex-1 items-center gap-0.5 text-[10px]"
      title={folded ? "Show the sounds on this bus" : "Hide the sounds on this bus"}
      onClick={onFold}
      disabled={members === 0}
    >
      {folded ? <ChevronRight className="size-3" /> : <ChevronLeft className="size-3" />}
      <span className="truncate">{members === 1 ? "1 sound" : `${members} sounds`}</span>
    </button>
  );
}

export function BusStrip({
  bus,
  mix,
  members,
  folded,
  onFold,
  result,
  frame,
  holdFrames,
}: {
  bus: Bus;
  mix: Mix;
  members: string[];
  folded: boolean;
  onFold: () => void;
  result: MixResult | null;
  frame: number;
  holdFrames: number;
}) {
  const levels = result?.levels.buses[bus.id];
  const update = (fn: (b: Bus) => Bus) => setMix((m) => updateBus(m, bus.id, fn));
  const binding: StripBinding = {
    controls: bus,
    target: { key: `bus:${bus.id}`, fraction: share(result, levels, frame) },
    update: (fn) => update((b) => ({ ...b, ...fn(b) })),
  };
  return (
    <StripShell className="bg-muted/30">
      <div className="flex w-full items-center gap-0.5">
        <BusName bus={bus} onRename={(name) => update((b) => ({ ...b, name }))} />
        {!BUILT_IN_BUSES.has(bus.id) ? (
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label="Remove bus"
            onClick={() => setMix((m) => removeBus(m, bus.id))}
          >
            <Trash2 />
          </Button>
        ) : null}
      </div>
      <div className="flex w-full items-center gap-0.5">
        <FoldButton folded={folded} members={members.length} onFold={onFold} />
        <BusMenu
          sounds={members.length}
          onBus={update}
          onSounds={(fn) => setMix((m) => updateChannels(m, members, fn))}
        />
      </div>
      <DuckPopover
        bus={bus}
        buses={mix.buses}
        onChange={bindChange((duck) => update((b) => ({ ...b, duck })))}
      />
      <div className="flex w-full justify-center">
        <HighpassKnob {...binding} />
      </div>
      <StripTone {...binding} reduction={null} />
      <StripLevel
        {...binding}
        label={`${bus.name} fader`}
        level={levelAt(levels, frame, holdFrames)}
        reduction={null}
      />
    </StripShell>
  );
}
