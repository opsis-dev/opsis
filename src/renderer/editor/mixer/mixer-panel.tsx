import {
  AudioLines,
  Clapperboard,
  Ellipsis,
  Music,
  Plus,
  SlidersVertical,
  Trash2,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { busOf, isAudible, soloActive, stripOf } from "@/shared/mix/levels";
import type { ChannelPlan } from "@/shared/mix/plan";
import type { MixResult } from "@/shared/mix/result";
import {
  BUILT_IN_BUSES,
  DEFAULT_MASTER,
  DEFAULT_STRIP,
  LOUDNESS_TARGETS,
  MIN_DB,
  STRIP_PRESETS,
  busSchema,
  type Bus,
  type Comp,
  type Eq,
  type EqBand,
  type LoudnessTarget,
  type Master,
  type Mix,
  type Strip,
} from "@/shared/mix/types";
import { getNodeDef } from "@/shared/nodes/registry";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import { Checkbox } from "@/renderer/components/ui/checkbox";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/renderer/components/ui/dropdown-menu";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/renderer/components/ui/empty";
import { Input } from "@/renderer/components/ui/input";
import { Label } from "@/renderer/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/renderer/components/ui/popover";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";
import { Spinner } from "@/renderer/components/ui/spinner";
import { Switch } from "@/renderer/components/ui/switch";
import { Toggle } from "@/renderer/components/ui/toggle";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { cn } from "@/renderer/lib/utils";
import { useAudioPreview, type PreviewFilter } from "../../store/audio-preview";
import { useMedia } from "../../store/media";
import { usePlayback } from "../../store/playback";
import { useUiStore } from "../../store/ui";
import { newId, useWorkflowStore } from "../../store/workflow";
import { useFilm } from "../film";
import { EqCurve } from "./eq-curve";
import { Knob, type DragPhase } from "./knob";
import { levelAt, Meter } from "./meter";

const db = (v: number) => (v <= MIN_DB ? "−∞" : `${v > 0 ? "+" : ""}${v.toFixed(1)}`);
const hz = (v: number) =>
  v >= 1000 ? `${(v / 1000).toFixed(v >= 10_000 ? 0 : 1)}k` : `${Math.round(v)}`;
const ms = (v: number) => (v >= 100 ? `${Math.round(v)}` : v.toFixed(1));
const panLabel = (v: number) =>
  Math.abs(v) < 0.01 ? "C" : v < 0 ? `L${Math.round(-v * 100)}` : `R${Math.round(v * 100)}`;

const BAND_LABELS: Record<EqBand["type"], string> = {
  lowshelf: "Low shelf",
  peak: "Bell",
  highshelf: "High shelf",
};

const KIND_ICON = { music: Music, effect: AudioLines, clip: Clapperboard } as const;

let clipboard: Partial<Strip> | null = null;

function gesture(phase: DragPhase, apply: () => void): void {
  const store = useWorkflowStore.getState();
  if (phase === "start") store.beginGesture();
  else if (phase === "move") apply();
  else store.endGesture();
}

const setMix = (fn: (mix: Mix) => Mix) => useWorkflowStore.getState().setMix(fn);

const updateStrip = (id: string, fn: (s: Strip) => Strip) =>
  setMix((mix) => ({
    ...mix,
    channels: { ...mix.channels, [id]: fn(mix.channels[id] ?? DEFAULT_STRIP) },
  }));

const updateBus = (id: string, fn: (b: Bus) => Bus) =>
  setMix((mix) => ({ ...mix, buses: mix.buses.map((b) => (b.id === id ? fn(b) : b)) }));

const updateMaster = (fn: (m: Master) => Master) =>
  setMix((mix) => ({ ...mix, master: fn(mix.master) }));

type Contribution = { gainDb?: number; pan?: number; filters?: PreviewFilter[] };
const contributions = new Map<string, Contribution>();
const baselines = new Map<string, number>();

function publishPreview(): void {
  let gainDb = 0;
  let pan = 0;
  const filters: PreviewFilter[] = [];
  for (const c of contributions.values()) {
    gainDb += c.gainDb ?? 0;
    pan += c.pan ?? 0;
    filters.push(...(c.filters ?? []));
  }
  useAudioPreview.getState().set({ gainDb, pan, filters: filters.slice(0, 6) });
}

function previewChange(
  id: string,
  current: number,
  next: number,
  make: (delta: number) => Contribution,
): void {
  if (!baselines.has(id)) baselines.set(id, current);
  const delta = next - baselines.get(id)!;
  contributions.set(id, make(delta));
  publishPreview();
}

function share(
  result: MixResult | null,
  levels: { rms: number[] } | undefined,
  frame: number,
): number {
  const master = result?.levels.master;
  if (!master || !levels) return 1;
  const f = Math.max(0, Math.min(master.rms.length - 1, Math.round(frame)));
  const pc = 10 ** ((levels.rms[f] ?? -120) / 10);
  const pm = 10 ** ((master.rms[f] ?? -120) / 10);
  return pm <= 1e-12 ? 1 : Math.min(1, pc / pm);
}

function gainContribution(fraction: number, deltaDb: number): Contribution {
  const p = Math.max(1e-9, 1 - fraction + fraction * 10 ** (deltaDb / 10));
  return { gainDb: 10 * Math.log10(p) };
}

function nodeName(nodeId: string): string {
  const wf = useWorkflowStore.getState().workflow;
  const node = wf?.nodes.find((n) => n.id === nodeId);
  if (!node) return "Sound";
  return node.label?.trim() || getNodeDef(node.type)?.title || "Sound";
}

export function MixerPanel() {
  const film = useFilm();
  const mix = film.mix;
  const plan = film.plan;
  const result = film.result;
  const unavailable = useMedia((s) => s.unavailable);
  const mixError = useMedia((s) => s.mixError);
  const frame = usePlayback((s) => s.frame);
  const fps = film.flat?.fps ?? 30;
  const holdFrames = Math.round(fps * 1.5);

  useEffect(() => {
    if (!film.stale) {
      contributions.clear();
      baselines.clear();
    }
  }, [film.stale]);

  const channels = plan?.channels ?? [];
  if (!film.flat || channels.length === 0) {
    return (
      <Empty className="h-full">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <SlidersVertical />
          </EmptyMedia>
          <EmptyTitle>No sound in this film</EmptyTitle>
          <EmptyDescription>
            Add music, a sound effect or a clip with sound, and its channel appears here.
          </EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  }

  const soloing = soloActive(mix);
  return (
    <div className="flex h-full flex-col">
      <div className="flex h-8 shrink-0 items-center gap-2 border-b px-2 text-xs">
        {unavailable ? (
          <span className="text-muted-foreground">
            Set up the engine in Settings to hear the full mix and see levels.
          </span>
        ) : film.failed && mixError ? (
          <span className="text-destructive truncate">{mixError}</span>
        ) : film.stale ? (
          <span className="text-muted-foreground flex items-center gap-1">
            <Spinner className="size-3" />
            Updating mix
          </span>
        ) : (
          <span className="text-muted-foreground">Levels show what the export plays.</span>
        )}
        {soloing ? <Badge variant="outline">Solo on</Badge> : null}
        <div className="flex-1" />
        <Button
          variant="ghost"
          size="sm"
          onClick={() =>
            setMix((m) => ({
              ...m,
              buses: [
                ...m.buses,
                busSchema.parse({ id: newId("bus"), name: `Bus ${m.buses.length + 1}` }),
              ],
            }))
          }
        >
          <Plus data-icon="inline-start" />
          Add bus
        </Button>
      </div>
      <div className="flex min-h-0 flex-1 overflow-x-auto overflow-y-auto">
        <div className="flex min-h-full gap-px">
          {channels.map((c) => (
            <ChannelStrip
              key={c.id}
              channel={c}
              mix={mix}
              result={result}
              frame={frame}
              holdFrames={holdFrames}
              sourceNodeId={film.sourceNodeId}
            />
          ))}
          <div className="bg-border mx-1 w-px shrink-0" />
          {mix.buses.map((b) => (
            <BusStrip
              key={b.id}
              bus={b}
              mix={mix}
              result={result}
              frame={frame}
              holdFrames={holdFrames}
            />
          ))}
        </div>
        <div className="flex-1" />
        <MasterStrip master={mix.master} result={result} frame={frame} holdFrames={holdFrames} />
      </div>
    </div>
  );
}

function StripShell({
  children,
  className,
  dim,
}: {
  children: React.ReactNode;
  className?: string;
  dim?: boolean;
}) {
  return (
    <div
      className={cn(
        "bg-card/40 flex w-[104px] shrink-0 flex-col items-center gap-1 px-1.5 py-1.5 [&>*]:shrink-0",
        dim && "opacity-60",
        className,
      )}
    >
      {children}
    </div>
  );
}

function MuteSolo({
  mute,
  solo,
  onMute,
  onSolo,
}: {
  mute: boolean;
  solo: boolean;
  onMute: (v: boolean) => void;
  onSolo: (v: boolean) => void;
}) {
  return (
    <div className="flex gap-1">
      <Tooltip>
        <TooltipTrigger
          render={
            <Toggle
              size="sm"
              variant="outline"
              className="data-[pressed]:bg-destructive/30 h-6 min-w-7 px-1 text-[11px]"
              pressed={mute}
              aria-label="Mute"
              onPressedChange={onMute}
            />
          }
        >
          M
        </TooltipTrigger>
        <TooltipContent>Mute. Saved with the project and left out of exports.</TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger
          render={
            <Toggle
              size="sm"
              variant="outline"
              className="data-[pressed]:bg-meter-mid/40 h-6 min-w-7 px-1 text-[11px]"
              pressed={solo}
              aria-label="Solo"
              onPressedChange={onSolo}
            />
          }
        >
          S
        </TooltipTrigger>
        <TooltipContent>
          Solo. While anything is soloed, only soloed sounds play, in exports too.
        </TooltipContent>
      </Tooltip>
    </div>
  );
}

function Fader({
  value,
  onChange,
  label,
}: {
  value: number;
  onChange: (v: number, phase: DragPhase) => void;
  label: string;
}) {
  const track = useRef<HTMLDivElement>(null);
  const t = (Math.max(MIN_DB, Math.min(12, value)) - MIN_DB) / (12 - MIN_DB);
  const fromY = (clientY: number) => {
    const rect = track.current!.getBoundingClientRect();
    const k = 1 - (clientY - rect.top) / Math.max(1, rect.height);
    return Math.round((MIN_DB + Math.min(1, Math.max(0, k)) * (12 - MIN_DB)) * 2) / 2;
  };
  const step = (dir: number, big: boolean) => {
    onChange(value, "start");
    onChange(Math.min(12, Math.max(MIN_DB, value + dir * (big ? 6 : 0.5))), "move");
    onChange(value, "end");
  };
  return (
    <div
      ref={track}
      role="slider"
      tabIndex={0}
      aria-label={label}
      aria-valuemin={MIN_DB}
      aria-valuemax={12}
      aria-valuenow={value}
      aria-valuetext={`${db(value)} dB`}
      title={`${label}: ${db(value)} dB. Double-click for 0 dB.`}
      className="focus-visible:ring-ring relative h-full w-4 cursor-ns-resize touch-none rounded outline-none focus-visible:ring-2"
      onPointerDown={(e) => {
        if (e.button !== 0) return;
        e.currentTarget.setPointerCapture(e.pointerId);
        onChange(value, "start");
        onChange(fromY(e.clientY), "move");
      }}
      onPointerMove={(e) => {
        if (e.currentTarget.hasPointerCapture(e.pointerId)) onChange(fromY(e.clientY), "move");
      }}
      onPointerUp={(e) => {
        if (!e.currentTarget.hasPointerCapture(e.pointerId)) return;
        e.currentTarget.releasePointerCapture(e.pointerId);
        onChange(value, "end");
      }}
      onDoubleClick={() => {
        onChange(value, "start");
        onChange(0, "move");
        onChange(0, "end");
      }}
      onKeyDown={(e) => {
        if (e.key === "ArrowUp") step(1, e.shiftKey);
        else if (e.key === "ArrowDown") step(-1, e.shiftKey);
        else return;
        e.preventDefault();
        e.stopPropagation();
      }}
    >
      <div className="bg-muted absolute inset-y-1 left-1/2 w-1 -translate-x-1/2 rounded-full" />
      <div
        className="bg-foreground/40 absolute left-0 h-px w-full"
        style={{ bottom: `${((0 - MIN_DB) / (12 - MIN_DB)) * 100}%` }}
      />
      <div
        className="bg-primary border-background absolute left-1/2 h-2.5 w-4 -translate-x-1/2 translate-y-1/2 rounded-sm border"
        style={{ bottom: `${t * 100}%` }}
      />
    </div>
  );
}

function ChannelStrip({
  channel,
  mix,
  result,
  frame,
  holdFrames,
  sourceNodeId,
}: {
  channel: ChannelPlan;
  mix: Mix;
  result: MixResult | null;
  frame: number;
  holdFrames: number;
  sourceNodeId: string | null;
}) {
  const strip = stripOf(mix, channel.id);
  const bus = busOf(mix, channel.id, channel.kind);
  const audible = isAudible(mix, channel.id, channel.kind);
  const levels = result?.levels.channels[channel.id];
  const level = levelAt(levels, frame, holdFrames);
  const reduction = result?.reduction.channels[channel.id];
  const Icon = KIND_ICON[channel.kind];
  const selected = useUiStore((s) => s.selectedIds.includes(channel.id));
  const set = (fn: (s: Strip) => Strip) => updateStrip(channel.id, fn);
  const fraction = share(result, levels, frame);
  const key = (name: string) => `ch:${channel.id}:${name}`;
  return (
    <StripShell dim={!audible}>
      <button
        type="button"
        className={cn(
          "flex w-full items-center gap-1 truncate rounded px-1 text-left text-xs font-medium",
          selected && "bg-accent",
        )}
        title={nodeName(channel.id)}
        onClick={() => useUiStore.getState().selectPinned([channel.id], sourceNodeId)}
      >
        <Icon className="text-muted-foreground size-3 shrink-0" />
        <span className="truncate">{nodeName(channel.id)}</span>
      </button>
      <div className="flex w-full items-center gap-0.5">
        <Select value={bus.id} onValueChange={(v) => set((s) => ({ ...s, bus: v as string }))}>
          <SelectTrigger
            size="sm"
            className="h-6 min-w-0 flex-1 px-1.5 text-[11px]"
            aria-label="Send to bus"
          >
            <SelectValue>
              {(v: string) => mix.buses.find((b) => b.id === v)?.name ?? "Bus"}
            </SelectValue>
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {mix.buses.map((b) => (
                <SelectItem key={b.id} value={b.id}>
                  {b.name}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
        <StripMenu
          onPreset={(p) => set((s) => ({ ...s, ...p }))}
          onCopy={() => {
            const { bus: _b, mute: _m, solo: _s, ...rest } = strip;
            clipboard = rest;
          }}
          onReset={() => set((s) => ({ ...DEFAULT_STRIP, bus: s.bus, mute: s.mute, solo: s.solo }))}
        />
      </div>
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
          onChange={(v, phase) =>
            gesture(phase, () => {
              previewChange(key("trim"), strip.trim, v, (d) => gainContribution(fraction, d));
              set((s) => ({ ...s, trim: v }));
            })
          }
        />
        <Knob
          label="Hz"
          size={24}
          value={strip.highpass.freq}
          min={20}
          max={1000}
          step={1}
          reset={80}
          log
          format={hz}
          disabled={!strip.highpass.on}
          onChange={(v, phase) =>
            gesture(phase, () => {
              contributions.set(key("hp"), {
                filters: [{ type: "highpass", freq: v, gain: 0, q: 0.7 }],
              });
              publishPreview();
              set((s) => ({ ...s, highpass: { ...s.highpass, freq: v } }));
            })
          }
        />
        <Knob
          label="Pan"
          size={24}
          value={strip.pan}
          min={-1}
          max={1}
          step={0.01}
          reset={0}
          format={panLabel}
          onChange={(v, phase) =>
            gesture(phase, () => {
              previewChange(key("pan"), strip.pan, v, (d) => ({ pan: d * fraction }));
              set((s) => ({ ...s, pan: v }));
            })
          }
        />
      </div>
      <EqPopover
        eq={strip.eq}
        highpass={strip.highpass.on ? strip.highpass.freq : null}
        onChange={(eq, phase, band) =>
          gesture(phase, () => {
            if (band !== undefined) {
              const b = eq.bands[band];
              const before = strip.eq.bands[band];
              previewChange(key(`eq${band}`), before.gain, b.gain, (d) => ({
                filters: [
                  {
                    type: b.type === "peak" ? "peaking" : b.type,
                    freq: b.freq,
                    gain: d * fraction,
                    q: b.q,
                  },
                ],
              }));
            }
            set((s) => ({ ...s, eq }));
          })
        }
      />
      <div className="flex w-full items-center gap-1">
        <Toggle
          size="sm"
          variant="outline"
          className="h-5 min-w-0 px-1 text-[10px]"
          pressed={strip.highpass.on}
          aria-label="Low cut"
          title="Cut rumble below the low-cut frequency"
          onPressedChange={(on) => set((s) => ({ ...s, highpass: { ...s.highpass, on } }))}
        >
          Low cut
        </Toggle>
        <CompPopover
          comp={strip.comp}
          reduction={reduction ? (reduction[Math.round(frame)] ?? 0) : null}
          onChange={(comp, phase) => gesture(phase, () => set((s) => ({ ...s, comp })))}
        />
      </div>
      <div className="flex min-h-16 flex-1 shrink items-stretch gap-2 py-0.5">
        <Fader
          label="Fader"
          value={strip.fader}
          onChange={(v, phase) =>
            gesture(phase, () => {
              previewChange(key("fader"), strip.fader, v, (d) => gainContribution(fraction, d));
              set((s) => ({ ...s, fader: v }));
            })
          }
        />
        <Meter
          peak={level.peak}
          rms={level.rms}
          hold={level.hold}
          reduction={strip.comp.on && reduction ? (reduction[Math.round(frame)] ?? 0) : undefined}
        />
      </div>
      <span className="text-[11px] tabular-nums">{db(strip.fader)} dB</span>
      <MuteSolo
        mute={strip.mute}
        solo={strip.solo}
        onMute={(mute) => set((s) => ({ ...s, mute }))}
        onSolo={(solo) => set((s) => ({ ...s, solo }))}
      />
    </StripShell>
  );
}

function StripMenu({
  onPreset,
  onCopy,
  onReset,
}: {
  onPreset: (p: Partial<Strip>) => void;
  onCopy: () => void;
  onReset: () => void;
}) {
  const [, force] = useState(0);
  return (
    <DropdownMenu onOpenChange={() => force((n) => n + 1)}>
      <DropdownMenuTrigger
        render={<Button variant="ghost" size="icon-xs" aria-label="Strip settings" />}
      >
        <Ellipsis />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-48">
        <DropdownMenuGroup>
          <DropdownMenuLabel>Presets</DropdownMenuLabel>
          {STRIP_PRESETS.map((p) => (
            <DropdownMenuItem key={p.id} onClick={() => onPreset(p.strip)}>
              {p.label}
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuItem onClick={onCopy}>Copy settings</DropdownMenuItem>
          <DropdownMenuItem disabled={!clipboard} onClick={() => clipboard && onPreset(clipboard)}>
            Paste settings
          </DropdownMenuItem>
          <DropdownMenuItem onClick={onReset}>Reset</DropdownMenuItem>
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function EqPopover({
  eq,
  highpass,
  onChange,
}: {
  eq: Eq;
  highpass: number | null;
  onChange: (eq: Eq, phase: DragPhase, band?: number) => void;
}) {
  const setBand = (i: number, patch: Partial<EqBand>, phase: DragPhase) =>
    onChange(
      { ...eq, bands: eq.bands.map((b, k) => (k === i ? { ...b, ...patch } : b)) },
      phase,
      i,
    );
  return (
    <Popover>
      <PopoverTrigger
        render={
          <button
            type="button"
            className="flex flex-col items-center gap-0.5"
            aria-label="Equaliser"
            title="Equaliser"
          />
        }
      >
        <EqCurve bands={eq.bands} on={eq.on} width={88} height={20} highpass={highpass} />
        <span className="text-muted-foreground text-[10px] leading-none">EQ</span>
      </PopoverTrigger>
      <PopoverContent className="w-[420px]">
        <div className="flex items-center justify-between">
          <Label className="text-sm">Equaliser</Label>
          <Switch
            checked={eq.on}
            aria-label="Equaliser on"
            onCheckedChange={(on) => {
              onChange({ ...eq, on }, "start");
              onChange({ ...eq, on }, "move");
              onChange({ ...eq, on }, "end");
            }}
          />
        </div>
        <EqCurve bands={eq.bands} on={eq.on} width={388} height={90} highpass={highpass} />
        <div className="grid grid-cols-4 gap-2">
          {eq.bands.map((band, i) => (
            <div key={i} className="flex flex-col items-center gap-1">
              <Select
                value={band.type}
                onValueChange={(v) => {
                  setBand(i, { type: v as EqBand["type"] }, "start");
                  setBand(i, { type: v as EqBand["type"] }, "move");
                  setBand(i, { type: v as EqBand["type"] }, "end");
                }}
              >
                <SelectTrigger
                  size="sm"
                  className="h-6 w-full px-1.5 text-[11px]"
                  aria-label="Band shape"
                >
                  <SelectValue>{(v: string) => BAND_LABELS[v as EqBand["type"]]}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {(Object.keys(BAND_LABELS) as EqBand["type"][]).map((t) => (
                      <SelectItem key={t} value={t}>
                        {BAND_LABELS[t]}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
              <Knob
                label="Freq"
                value={band.freq}
                min={20}
                max={20_000}
                step={1}
                reset={band.freq}
                log
                format={hz}
                disabled={!eq.on}
                onChange={(v, p) => setBand(i, { freq: v }, p)}
              />
              <Knob
                label="Gain"
                value={band.gain}
                min={-18}
                max={18}
                step={0.5}
                reset={0}
                format={db}
                disabled={!eq.on}
                onChange={(v, p) => setBand(i, { gain: v }, p)}
              />
              <Knob
                label="Width"
                value={band.q}
                min={0.1}
                max={10}
                step={0.05}
                reset={1}
                log
                format={(v) => v.toFixed(2)}
                disabled={!eq.on}
                onChange={(v, p) => setBand(i, { q: v }, p)}
              />
            </div>
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}

function CompPopover({
  comp,
  reduction,
  onChange,
  title = "Compressor",
}: {
  comp: Comp;
  reduction: number | null;
  onChange: (comp: Comp, phase: DragPhase) => void;
  title?: string;
}) {
  const set = (patch: Partial<Comp>, phase: DragPhase) => onChange({ ...comp, ...patch }, phase);
  return (
    <Popover>
      <PopoverTrigger
        render={
          <button
            type="button"
            className={cn(
              "flex h-5 min-w-0 flex-1 items-center justify-between rounded border px-1.5 text-[10px]",
              comp.on ? "border-keyframe/60" : "text-muted-foreground",
            )}
            aria-label={title}
          />
        }
      >
        <span>Comp</span>
        {comp.on && reduction !== null ? (
          <span className="tabular-nums">−{reduction.toFixed(1)}</span>
        ) : null}
      </PopoverTrigger>
      <PopoverContent className="w-80">
        <div className="flex items-center justify-between">
          <Label className="text-sm">{title}</Label>
          <Switch
            checked={comp.on}
            aria-label={`${title} on`}
            onCheckedChange={(on) => {
              set({ on }, "start");
              set({ on }, "move");
              set({ on }, "end");
            }}
          />
        </div>
        <div className="grid grid-cols-5 gap-1">
          <Knob
            label="Threshold"
            value={comp.threshold}
            min={-60}
            max={0}
            step={0.5}
            reset={-18}
            format={db}
            disabled={!comp.on}
            onChange={(v, p) => set({ threshold: v }, p)}
          />
          <Knob
            label="Ratio"
            value={comp.ratio}
            min={1}
            max={20}
            step={0.1}
            reset={3}
            log
            format={(v) => `${v.toFixed(1)}:1`}
            disabled={!comp.on}
            onChange={(v, p) => set({ ratio: v }, p)}
          />
          <Knob
            label="Attack"
            value={comp.attack}
            min={0.1}
            max={500}
            step={0.1}
            reset={10}
            log
            format={ms}
            disabled={!comp.on}
            onChange={(v, p) => set({ attack: v }, p)}
          />
          <Knob
            label="Release"
            value={comp.release}
            min={5}
            max={3000}
            step={1}
            reset={150}
            log
            format={ms}
            disabled={!comp.on}
            onChange={(v, p) => set({ release: v }, p)}
          />
          <Knob
            label="Makeup"
            value={comp.makeup}
            min={0}
            max={24}
            step={0.5}
            reset={0}
            format={db}
            disabled={!comp.on}
            onChange={(v, p) => set({ makeup: v }, p)}
          />
        </div>
        <p className="text-muted-foreground text-xs">Attack and release are in milliseconds.</p>
      </PopoverContent>
    </Popover>
  );
}

function DuckPopover({ bus, buses }: { bus: Bus; buses: Bus[] }) {
  const set = (patch: Partial<Bus["duck"]>) =>
    updateBus(bus.id, (b) => ({ ...b, duck: { ...b.duck, ...patch } }));
  const knob = (patch: (v: number) => Partial<Bus["duck"]>) => (v: number, phase: DragPhase) =>
    gesture(phase, () => set(patch(v)));
  return (
    <Popover>
      <PopoverTrigger
        render={
          <button
            type="button"
            className={cn(
              "w-full rounded border px-1.5 py-0.5 text-[10px]",
              bus.duck.on ? "border-keyframe/60" : "text-muted-foreground",
            )}
          />
        }
      >
        {bus.duck.on ? "Ducks" : "No ducking"}
      </PopoverTrigger>
      <PopoverContent className="w-80">
        <div className="flex items-center justify-between">
          <Label className="text-sm">Lower this bus while others play</Label>
          <Switch
            checked={bus.duck.on}
            aria-label="Ducking on"
            onCheckedChange={(on) => set({ on })}
          />
        </div>
        <div className="flex flex-col gap-1.5">
          <span className="text-muted-foreground text-xs">Listen to</span>
          {buses
            .filter((b) => b.id !== bus.id)
            .map((b) => (
              <Label key={b.id} className="flex items-center gap-2 text-sm font-normal">
                <Checkbox
                  checked={bus.duck.keys.includes(b.id)}
                  onCheckedChange={(on) =>
                    set({
                      keys: on ? [...bus.duck.keys, b.id] : bus.duck.keys.filter((k) => k !== b.id),
                    })
                  }
                />
                {b.name}
              </Label>
            ))}
        </div>
        <div className="grid grid-cols-3 gap-1">
          <Knob
            label="Amount"
            value={bus.duck.amount}
            min={1}
            max={24}
            step={0.5}
            reset={10}
            format={(v) => `${v.toFixed(1)} dB`}
            disabled={!bus.duck.on}
            onChange={knob((v) => ({ amount: v }))}
          />
          <Knob
            label="Attack"
            value={bus.duck.attack}
            min={5}
            max={2000}
            step={1}
            reset={80}
            log
            format={ms}
            disabled={!bus.duck.on}
            onChange={knob((v) => ({ attack: v }))}
          />
          <Knob
            label="Release"
            value={bus.duck.release}
            min={20}
            max={5000}
            step={1}
            reset={400}
            log
            format={ms}
            disabled={!bus.duck.on}
            onChange={knob((v) => ({ release: v }))}
          />
        </div>
        <p className="text-muted-foreground text-xs">
          Louder sounds lower it more. Attack and release are in milliseconds.
        </p>
      </PopoverContent>
    </Popover>
  );
}

function BusStrip({
  bus,
  mix,
  result,
  frame,
  holdFrames,
}: {
  bus: Bus;
  mix: Mix;
  result: MixResult | null;
  frame: number;
  holdFrames: number;
}) {
  const [editing, setEditing] = useState(false);
  const levels = result?.levels.buses[bus.id];
  const level = levelAt(levels, frame, holdFrames);
  const fraction = share(result, levels, frame);
  const set = (fn: (b: Bus) => Bus) => updateBus(bus.id, fn);
  return (
    <StripShell className="bg-muted/30">
      <div className="flex w-full items-center gap-0.5">
        {editing ? (
          <Input
            autoFocus
            defaultValue={bus.name}
            className="h-6 px-1 text-xs"
            onKeyDown={(e) => {
              e.stopPropagation();
              if (e.key === "Enter" || e.key === "Escape") (e.target as HTMLInputElement).blur();
            }}
            onBlur={(e) => {
              setEditing(false);
              const name = e.target.value.trim();
              if (name && name !== bus.name) set((b) => ({ ...b, name }));
            }}
          />
        ) : (
          <button
            type="button"
            className="flex-1 truncate px-1 text-left text-xs font-medium"
            title="Double-click to rename"
            onDoubleClick={() => setEditing(true)}
          >
            {bus.name}
          </button>
        )}
        {!BUILT_IN_BUSES.has(bus.id) ? (
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label="Remove bus"
            onClick={() =>
              setMix((m) => ({
                ...m,
                buses: m.buses
                  .filter((b) => b.id !== bus.id)
                  .map((b) => ({
                    ...b,
                    duck: { ...b.duck, keys: b.duck.keys.filter((k) => k !== bus.id) },
                  })),
                channels: Object.fromEntries(
                  Object.entries(m.channels).map(([id, s]) => [
                    id,
                    s.bus === bus.id ? { ...s, bus: undefined } : s,
                  ]),
                ),
              }))
            }
          >
            <Trash2 />
          </Button>
        ) : null}
      </div>
      <span className="text-muted-foreground text-[10px]">Bus</span>
      <DuckPopover bus={bus} buses={mix.buses} />
      <div className="flex min-h-16 flex-1 shrink items-stretch gap-2 py-0.5">
        <Fader
          label={`${bus.name} fader`}
          value={bus.fader}
          onChange={(v, phase) =>
            gesture(phase, () => {
              previewChange(`bus:${bus.id}`, bus.fader, v, (d) => gainContribution(fraction, d));
              set((b) => ({ ...b, fader: v }));
            })
          }
        />
        <Meter peak={level.peak} rms={level.rms} hold={level.hold} />
      </div>
      <span className="text-[11px] tabular-nums">{db(bus.fader)} dB</span>
      <MuteSolo
        mute={bus.mute}
        solo={bus.solo}
        onMute={(mute) => set((b) => ({ ...b, mute }))}
        onSolo={(solo) => set((b) => ({ ...b, solo }))}
      />
    </StripShell>
  );
}

const LOUDNESS_LABELS: Record<LoudnessTarget, string> = {
  off: "Off",
  web: "Online video (−14)",
  podcast: "Podcast (−16)",
  broadcast: "Broadcast (−23)",
};

function Readout({ label, value, unit }: { label: string; value: number | null; unit: string }) {
  return (
    <div className="flex w-full items-baseline justify-between text-[10px]">
      <span className="text-muted-foreground">{label}</span>
      <span className="tabular-nums">
        {value === null || value <= -70 ? "—" : value.toFixed(1)}{" "}
        <span className="text-muted-foreground">{unit}</span>
      </span>
    </div>
  );
}

function MasterStrip({
  master,
  result,
  frame,
  holdFrames,
}: {
  master: Master;
  result: MixResult | null;
  frame: number;
  holdFrames: number;
}) {
  const levels = result?.levels.master ?? undefined;
  const level = levelAt(levels, frame, holdFrames);
  const loud = result?.loudness;
  const f = Math.max(0, Math.round(frame));
  const reduction = result?.reduction.master;
  const set = (fn: (m: Master) => Master) => updateMaster(fn);
  const target = LOUDNESS_TARGETS[master.loudness];
  const loudnessMemo = useMemo(
    () => ({
      m: loud?.momentary[Math.min(f, (loud?.momentary.length ?? 1) - 1)] ?? null,
      s: loud?.shortTerm[Math.min(f, (loud?.shortTerm.length ?? 1) - 1)] ?? null,
    }),
    [loud, f],
  );
  return (
    <div className="bg-sidebar sticky right-0 flex w-[250px] shrink-0 gap-2 border-l px-2 py-1.5">
      <div className="flex min-w-0 flex-1 flex-col gap-1 [&>*]:shrink-0">
        <div className="flex w-full items-center justify-between">
          <span className="text-xs font-medium">Master</span>
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label="Reset master"
            title="Reset master"
            onClick={() => set(() => DEFAULT_MASTER)}
          >
            <Ellipsis />
          </Button>
        </div>
        <EqPopover
          eq={master.eq}
          highpass={null}
          onChange={(eq, phase, band) =>
            gesture(phase, () => {
              if (band !== undefined) {
                const b = eq.bands[band];
                previewChange(`master:eq${band}`, master.eq.bands[band].gain, b.gain, (d) => ({
                  filters: [
                    { type: b.type === "peak" ? "peaking" : b.type, freq: b.freq, gain: d, q: b.q },
                  ],
                }));
              }
              set((m) => ({ ...m, eq }));
            })
          }
        />
        <div className="flex w-full">
          <CompPopover
            comp={master.comp}
            reduction={reduction ? (reduction[f] ?? 0) : null}
            onChange={(comp, phase) => gesture(phase, () => set((m) => ({ ...m, comp })))}
          />
        </div>
        <div className="flex w-full items-center justify-between gap-1">
          <Label className="text-[10px] font-normal">Limiter</Label>
          <Switch
            size="sm"
            checked={master.limiter.on}
            aria-label="Limiter"
            onCheckedChange={(on) => set((m) => ({ ...m, limiter: { ...m.limiter, on } }))}
          />
          <Knob
            label="Ceiling"
            size={24}
            value={master.limiter.ceiling}
            min={-12}
            max={0}
            step={0.1}
            reset={-1}
            format={db}
            disabled={!master.limiter.on}
            onChange={(v, phase) =>
              gesture(phase, () => set((m) => ({ ...m, limiter: { ...m.limiter, ceiling: v } })))
            }
          />
        </div>
        <div className="flex w-full flex-col gap-0.5">
          <span className="text-muted-foreground text-[10px]">Loudness target</span>
          <Select
            value={master.loudness}
            onValueChange={(v) => set((m) => ({ ...m, loudness: v as LoudnessTarget }))}
          >
            <SelectTrigger
              size="sm"
              className="h-6 w-full px-1.5 text-[11px]"
              aria-label="Loudness target"
            >
              <SelectValue>{(v: string) => LOUDNESS_LABELS[v as LoudnessTarget]}</SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {(Object.keys(LOUDNESS_LABELS) as LoudnessTarget[]).map((t) => (
                  <SelectItem key={t} value={t}>
                    {LOUDNESS_LABELS[t]}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>
        <div className="flex w-full flex-col gap-0.5 border-t pt-1">
          <Readout label="Momentary" value={loudnessMemo.m} unit="LUFS" />
          <Readout label="Short-term" value={loudnessMemo.s} unit="LUFS" />
          <Readout label="Integrated" value={loud?.integrated ?? null} unit="LUFS" />
          <Readout label="True peak" value={loud?.truePeak ?? null} unit="dBTP" />
          <Readout label="Range" value={loud?.range ?? null} unit="LU" />
          {target ? (
            <span className="text-muted-foreground text-[10px]">Aim: {target.lufs} LUFS</span>
          ) : null}
        </div>
      </div>
      <div className="flex w-12 shrink-0 flex-col items-center gap-1">
        <div className="flex min-h-16 w-full flex-1 items-stretch justify-center gap-2 py-0.5">
          <Fader
            label="Master fader"
            value={master.fader}
            onChange={(v, phase) =>
              gesture(phase, () => {
                previewChange("master:fader", master.fader, v, (d) => ({ gainDb: d }));
                set((m) => ({ ...m, fader: v }));
              })
            }
          />
          <Meter
            peak={level.peak}
            rms={level.rms}
            hold={level.hold}
            reduction={master.comp.on && reduction ? (reduction[f] ?? 0) : undefined}
          />
        </div>
        <span className="text-[11px] whitespace-nowrap tabular-nums">{db(master.fader)} dB</span>
      </div>
    </div>
  );
}
