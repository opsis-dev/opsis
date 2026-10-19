import { AudioLines, Clapperboard, Music } from "lucide-react";
import { useShallow } from "zustand/react/shallow";
import { cn } from "@/renderer/lib/utils";
import { findById } from "@/shared/lists";
import { updateChannels } from "@/shared/mix/edits";
import { audibility, busOf, stripOf } from "@/shared/mix/levels";
import type { ChannelPlan } from "@/shared/mix/plan";
import type { MixResult } from "@/shared/mix/result";
import { PAN_RANGE, TRIM_RANGE, type Bus, type Mix, type Strip } from "@/shared/mix/types";
import { nodeNameById } from "@/shared/nodes/registry";
import { useUiStore } from "../../store/ui";
import { useWorkflowStore } from "../../store/workflow";
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

const useChannelName = (nodeId: string): string =>
  useWorkflowStore((s) =>
    s.workflow ? nodeNameById(s.workflow.nodes, nodeId, CHANNEL_FALLBACK) : CHANNEL_FALLBACK,
  );

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
  update: (fn: (s: Strip) => Strip) => void;
  binding: StripBinding;
}) {
  const { target } = binding;
  return (
    <div className="flex w-full justify-between">
      <Knob
        label="Trim"
        size={24}
        value={strip.trim}
        {...rangeProps(TRIM_RANGE)}
        step={0.5}
        format={formatDb}
        unit="dB"
        gesture={bindGain(target, "trim", strip.trim, (trim) => update((s) => ({ ...s, trim })))}
      />
      <HighpassKnob {...binding} />
      <Knob
        label="Pan"
        size={24}
        value={strip.pan}
        {...rangeProps(PAN_RANGE)}
        step={0.01}
        format={formatPan}
        gesture={bindPan(target, strip.pan, (pan) => update((s) => ({ ...s, pan })))}
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
  const setMix = useWorkflowStore((s) => s.setMix);
  const strip = stripOf(mix, channel.id);
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
  const update = (fn: (s: Strip) => Strip) => setMix((m) => updateChannels(m, [channel.id], fn));
  const target = previewTarget(`ch:${channel.id}`, result, levels);
  const binding = stripBinding(strip, target, update);
  const name = useChannelName(channel.id);
  return (
    <StripShell
      dim={!audibility(mix)(channel.id, channel.kind)}
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
          bus={busOf(mix, channel.id, channel.kind)}
          buses={mix.buses}
          onChange={(bus) => update((s) => ({ ...s, bus }))}
        />
        <StripMenu
          strip={strip}
          count={targets.length}
          onApply={(fn) => setMix((m) => updateChannels(m, targets, fn))}
        />
      </div>
      <ChannelKnobs strip={strip} update={update} binding={binding} />
      <StripTone {...binding} reduction={reduction} />
      <StripLevel {...binding} label="Fader" levels={levels} reduction={reduction} />
    </StripShell>
  );
}
