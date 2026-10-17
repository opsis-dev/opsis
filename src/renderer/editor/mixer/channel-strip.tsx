import { AudioLines, Clapperboard, Music } from "lucide-react";
import { useShallow } from "zustand/react/shallow";
import { updateChannels } from "@/shared/mix/edits";
import { busOf, isAudible, stripOf } from "@/shared/mix/levels";
import type { ChannelPlan } from "@/shared/mix/plan";
import type { MixResult } from "@/shared/mix/result";
import { PAN_RANGE, TRIM_RANGE, type Bus, type Mix, type Strip } from "@/shared/mix/types";
import { nodeNameById } from "@/shared/nodes/registry";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";
import { cn } from "@/renderer/lib/utils";
import { toggleIds, useUiStore } from "../../store/ui";
import { useWorkflowStore } from "../../store/workflow";
import { db, panLabel } from "./audio-format";
import { bindGain, bindPan, previewTarget } from "./bindings";
import { Knob } from "./knob";
import { rangeKnob } from "./knob-math";
import { useMixerState } from "./mixer-state";
import { StripMenu } from "./strip-menus";
import {
  HighpassKnob,
  StripLevel,
  StripShell,
  StripTone,
  stripBinding,
  type StripBinding,
} from "./strip-parts";
import { findById } from "@/shared/lists";

const KIND_ICON = { music: Music, effect: AudioLines, clip: Clapperboard } as const;

const CHANNEL_FALLBACK = "Sound";

const useChannelName = (nodeId: string): string =>
  useWorkflowStore((s) =>
    s.workflow ? nodeNameById(s.workflow, nodeId, CHANNEL_FALLBACK) : CHANNEL_FALLBACK,
  );

function pickStrip(
  id: string,
  order: string[],
  e: React.MouseEvent,
  sourceNodeId: string | null,
): void {
  const ui = useUiStore.getState();
  const { anchor, setAnchor } = useMixerState.getState();
  let next = [id];
  if (e.shiftKey && anchor && order.includes(anchor)) {
    const [a, b] = [order.indexOf(anchor), order.indexOf(id)].toSorted((x, y) => x - y);
    next = [...new Set([...ui.selectedIds, ...order.slice(a, b + 1)])];
  } else if (e.ctrlKey || e.metaKey) {
    next = toggleIds(ui.selectedIds, [id]);
  }
  if (!e.shiftKey) setAnchor(id);
  ui.selectPinned(next, sourceNodeId);
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
    <Select
      value={bus.id}
      onValueChange={(id) => {
        if (id) onChange(id);
      }}
    >
      <SelectTrigger
        size="sm"
        className="h-6 min-w-0 flex-1 px-1.5 text-[11px]"
        aria-label="Send to bus"
      >
        <SelectValue>{(v: string) => findById(buses, v)?.name ?? "Bus"}</SelectValue>
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          {buses.map((b) => (
            <SelectItem key={b.id} value={b.id}>
              {b.name}
            </SelectItem>
          ))}
        </SelectGroup>
      </SelectContent>
    </Select>
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
        {...rangeKnob(TRIM_RANGE)}
        step={0.5}
        format={db}
        onChange={bindGain(target, "trim", strip.trim, (trim) => update((s) => ({ ...s, trim })))}
      />
      <HighpassKnob {...binding} />
      <Knob
        label="Pan"
        size={24}
        value={strip.pan}
        {...rangeKnob(PAN_RANGE)}
        step={0.01}
        format={panLabel}
        onChange={bindPan(target, strip.pan, (pan) => update((s) => ({ ...s, pan })))}
      />
    </div>
  );
}

export function ChannelStrip({
  channel,
  order,
  mix,
  result,
  holdFrames,
  sourceNodeId,
}: {
  channel: ChannelPlan;
  order: string[];
  mix: Mix;
  result: MixResult | null;
  holdFrames: number;
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
      dim={!isAudible(mix, channel.id, channel.kind)}
      className={cn(group.length > 1 && "ring-primary/60 ring-1 ring-inset")}
    >
      <button
        type="button"
        className={cn(
          "flex w-full items-center gap-1 truncate rounded px-1 text-left text-xs font-medium",
          selected && "bg-accent",
        )}
        title={`${name}. Ctrl-click or Shift-click to select several strips.`}
        onClick={(e) => pickStrip(channel.id, order, e, sourceNodeId)}
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
      <StripLevel
        {...binding}
        label="Fader"
        levels={levels}
        reduction={reduction}
        holdFrames={holdFrames}
      />
    </StripShell>
  );
}
