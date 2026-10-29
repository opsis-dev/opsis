import { ChevronLeft, ChevronRight, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { Bus, BusPatch, Mix, MixResult } from "@/generated/bindings";
import { mixerBook } from "@/store/catalog";
import { workflowEdits } from "@/store/workflow";
import { useRenaming } from "../inline-edit";
import { bindChange, previewTarget } from "./bindings";
import { DuckPopover } from "./dynamics-popovers";
import { BusMenu } from "./strip-menus";
import { HighpassKnob, StripLevel, StripShell, StripTone, stripBinding } from "./strip-parts";

function BusName({ bus, onRename }: { bus: Bus; onRename: (name: string) => void }) {
  const { input, startRenaming } = useRenaming(bus.name, onRename, "h-6 px-1 text-xs");
  return (
    input ?? (
      <button
        type="button"
        className="flex-1 truncate px-1 text-left text-xs font-medium"
        title="Double-click to rename"
        onDoubleClick={startRenaming}
      >
        {bus.name}
      </button>
    )
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
}: {
  bus: Bus;
  mix: Mix;
  members: string[];
  folded: boolean;
  onFold: () => void;
  result: MixResult | null;
}) {
  const levels = result?.levels.buses[bus.id];
  const update = (patch: BusPatch) => workflowEdits.mix({ op: "bus", id: bus.id, patch });
  const binding = stripBinding(bus, previewTarget(`bus:${bus.id}`, result, levels), update);
  return (
    <StripShell className="bg-muted/30">
      <div className="flex w-full items-center gap-0.5">
        <BusName bus={bus} onRename={(name) => update({ name })} />
        {!mixerBook().builtInBuses.includes(bus.id) ? (
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label="Remove bus"
            onClick={() => workflowEdits.mix({ op: "removeBus", id: bus.id })}
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
          onBusReset={() => workflowEdits.mix({ op: "resetBus", id: bus.id })}
          onSounds={(patch) => workflowEdits.mix({ op: "channels", ids: members, patch })}
          onSoundsReset={() => workflowEdits.mix({ op: "resetChannels", ids: members })}
        />
      </div>
      <DuckPopover bus={bus} buses={mix.buses} gesture={bindChange((duck) => update({ duck }))} />
      <div className="flex w-full justify-center">
        <HighpassKnob {...binding} />
      </div>
      <StripTone {...binding} reduction={null} />
      <StripLevel {...binding} label={`${bus.name} fader`} levels={levels} reduction={null} />
    </StripShell>
  );
}
