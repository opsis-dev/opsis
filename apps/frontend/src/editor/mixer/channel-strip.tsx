import { findById } from "@opsis/video/common/lists";
import { AudioLines, Clapperboard, Music } from "lucide-react";
import { useShallow } from "zustand/react/shallow";
import type { Bus, ChannelPlan, Mix, MixResult, Strip, StripPatch } from "@/generated/bindings";
import { cn } from "@/lib/utils";
import { mixerBook } from "@/store/catalog";
import { nodeNameById } from "@/store/catalog";
import { useUiStore } from "@/store/ui";
import { useWorkflowStore, workflowEdits } from "@/store/workflow";
import { OptionSelect } from "../option-select";
import { formatDb, formatPan } from "./audio-format";
import { bindGain, bindPan, previewTarget } from "./bindings";
import { Knob } from "./knob";
import { rangeProps } from "./knob-math";
import { selectStrip } from "./mixer-state";
import { StripMenu } from "./strip-menus";
import {
  HighpassKnob,
  StripLevel,
  StripShell,
  StripTone,
  stripBinding,
  type StripBinding,
} from "./strip-parts";

const KIND_ICON = { music: Music, effect: AudioLines, clip: Clapperboard } as const;

const CHANNEL_FALLBACK = "Sound";

function useChannelName(nodeId: string): string {
  return useWorkflowStore((s) =>
    s.workflow ? nodeNameById(s.workflow.nodes, nodeId, CHANNEL_FALLBACK) : CHANNEL_FALLBACK,
  );
}

function BusSelect({
  bus,
  buses,
  onChange,
}: {
  bus: Bus;
  buses: Bus[];
  onChange: (id: string) => void;
}) {
  return (
    <OptionSelect
      label="Send to bus"
      value={bus.id}
      options={buses.map((b) => b.id)}
      optionLabel={(id) => findById(buses, id)?.name ?? "Bus"}
      className="h-6 min-w-0 flex-1 px-1.5 text-[11px]"
      onChange={onChange}
    />
  );
}

function ChannelKnobs({
  strip,
  update,
  binding,
}: {
  strip: Strip;
  update: (patch: StripPatch) => void;
  binding: StripBinding;
}) {
  const { target } = binding;
  return (
    <div className="flex w-full justify-between">
      <Knob
        label="Trim"
        size={24}
        value={strip.trim}
        {...rangeProps(mixerBook().trim)}
        step={0.5}
        format={formatDb}
        unit="dB"
        gesture={bindGain(target, "trim", strip.trim, (trim) => update({ trim }))}
      />
      <HighpassKnob {...binding} />
      <Knob
        label="Pan"
        size={24}
        value={strip.pan}
        {...rangeProps(mixerBook().pan)}
        step={0.01}
        format={formatPan}
        gesture={bindPan(target, strip.pan, (pan) => update({ pan }))}
      />
    </div>
  );
}

export function ChannelStrip({
  channel,
  order,
  mix,
  result,
  sourceNodeId,
}: {
  channel: ChannelPlan;
  order: string[];
  mix: Mix;
  result: MixResult | null;
  sourceNodeId: string | null;
}) {
  const strip = channel.strip;
  const levels = result?.levels.channels[channel.id];
  const reduction = result?.reduction.channels[channel.id];
  const Icon = KIND_ICON[channel.kind];
  const selected = useUiStore((s) => s.selectedIds.includes(channel.id));
  const group = useUiStore(
    useShallow((s) =>
      s.selectedIds.includes(channel.id) ? s.selectedIds.filter((id) => order.includes(id)) : [],
    ),
  );
  const targets = group.length > 1 ? group : [channel.id];
  function update(patch: StripPatch) {
    return workflowEdits.mix({ op: "channels", ids: [channel.id], patch });
  }
  const target = previewTarget(`ch:${channel.id}`, result, levels);
  const binding = stripBinding(strip, target, update);
  const name = useChannelName(channel.id);
  return (
    <StripShell
      dim={!channel.audible}
      className={cn(group.length > 1 && "ring-primary/60 ring-1 ring-inset")}
    >
      <button
        type="button"
        className={cn(
          "flex w-full items-center gap-1 truncate rounded px-1 text-left text-xs font-medium",
          selected && "bg-accent",
        )}
        title={`${name}. Ctrl-click or Shift-click to select several strips.`}
        onClick={(e) => selectStrip(channel.id, order, e, sourceNodeId)}
      >
        <Icon className="text-muted-foreground size-3 shrink-0" />
        <span className="truncate">{name}</span>
      </button>
      <div className="flex w-full items-center gap-0.5">
        <BusSelect
          bus={findById(mix.buses, channel.bus) ?? mix.buses[0]}
          buses={mix.buses}
          onChange={(bus) => update({ bus })}
        />
        <StripMenu
          strip={strip}
          count={targets.length}
          onApply={(patch) => workflowEdits.mix({ op: "channels", ids: targets, patch })}
          onReset={() => workflowEdits.mix({ op: "resetChannels", ids: targets })}
        />
      </div>
      <ChannelKnobs strip={strip} update={update} binding={binding} />
      <StripTone {...binding} reduction={reduction} />
      <StripLevel {...binding} label="Fader" levels={levels} reduction={reduction} />
    </StripShell>
  );
}
