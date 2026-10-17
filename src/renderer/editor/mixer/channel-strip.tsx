import { AudioLines, Clapperboard, Music } from "lucide-react";
import { useShallow } from "zustand/react/shallow";
import { updateChannels } from "@/shared/mix/edits";
import { busOf, isAudible, stripOf } from "@/shared/mix/levels";
import type { ChannelPlan } from "@/shared/mix/plan";
import type { MixResult } from "@/shared/mix/result";
import type { Bus, Mix, Strip } from "@/shared/mix/types";
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
import { useUiStore } from "../../store/ui";
import { useWorkflowStore } from "../../store/workflow";
import { db, panLabel } from "./audio-format";
import { bindGain, bindPan, setMix, share, type PreviewTarget } from "./bindings";
import { Knob } from "./knob";
import { levelAt, reductionAt } from "./meter";
import { useMixerState } from "./mixer-state";
import { StripMenu } from "./strip-menus";
import { HighpassKnob, StripLevel, StripShell, StripTone, type StripBinding } from "./strip-parts";

const KIND_ICON = { music: Music, effect: AudioLines, clip: Clapperboard } as const;

function channelName(nodeId: string): string {
  const wf = useWorkflowStore.getState().workflow;
  return wf ? nodeNameById(wf, nodeId, "Sound") : "Sound";
}

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
    next = ui.selectedIds.includes(id)
      ? ui.selectedIds.filter((x) => x !== id)
      : [...ui.selectedIds, id];
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
        <SelectValue>{(v: string) => buses.find((b) => b.id === v)?.name ?? "Bus"}</SelectValue>
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
  target,
  update,
  binding,
}: {
  strip: Strip;
  target: PreviewTarget;
  update: (fn: (s: Strip) => Strip) => void;
  binding: StripBinding;
}) {
  return (
    <div className="flex w-full justify-between">
      <Knob
        label="Trim"
        size={24}
        value={strip.trim}
        min={-24}
        max={24}
        step={0.5}
        reset={0}
        format={db}
        onChange={bindGain(target, "trim", strip.trim, (trim) => update((s) => ({ ...s, trim })))}
      />
      <HighpassKnob {...binding} />
      <Knob
        label="Pan"
        size={24}
        value={strip.pan}
        min={-1}
        max={1}
        step={0.01}
        reset={0}
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
  frame,
  holdFrames,
  sourceNodeId,
}: {
  channel: ChannelPlan;
  order: string[];
  mix: Mix;
  result: MixResult | null;
  frame: number;
  holdFrames: number;
  sourceNodeId: string | null;
}) {
  const strip = stripOf(mix, channel.id);
  const levels = result?.levels.channels[channel.id];
  const reduction = reductionAt(result?.reduction.channels[channel.id], frame);
  const Icon = KIND_ICON[channel.kind];
  const selected = useUiStore((s) => s.selectedIds.includes(channel.id));
  const group = useUiStore(
    useShallow((s) =>
      s.selectedIds.includes(channel.id) ? s.selectedIds.filter((id) => order.includes(id)) : [],
    ),
  );
  const targets = group.length > 1 ? group : [channel.id];
  const update = (fn: (s: Strip) => Strip) => setMix((m) => updateChannels(m, [channel.id], fn));
  const target = { key: `ch:${channel.id}`, fraction: share(result, levels, frame) };
  const binding: StripBinding = {
    controls: strip,
    target,
    update: (fn) => update((s) => ({ ...s, ...fn(s) })),
  };
  const name = channelName(channel.id);
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
      <ChannelKnobs strip={strip} target={target} update={update} binding={binding} />
      <StripTone {...binding} reduction={reduction} />
      <StripLevel
        {...binding}
        label="Fader"
        level={levelAt(levels, frame, holdFrames)}
        reduction={reduction}
      />
    </StripShell>
  );
}
