import { Plus, SlidersVertical } from "lucide-react";
import { useState } from "react";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/renderer/components/ui/empty";
import { Spinner } from "@/renderer/components/ui/spinner";
import { randomId } from "@/shared/ids";
import { addBus } from "@/shared/mix/edits";
import { soloActive } from "@/shared/mix/levels";
import { useMedia } from "../../store/media";
import { toggleIds } from "../../store/ui";
import { readPref, writePref } from "../../store/view-prefs";
import { useWorkflowStore } from "../../store/workflow";
import { useFilm, type Film } from "../film";
import { BusStrip } from "./bus-strip";
import { ChannelStrip } from "./channel-strip";
import { MasterStrip } from "./master-strip";

const HOLD_SECONDS = 1.5;

function useCollapsedBuses() {
  const [collapsed, setCollapsed] = useState<string[]>(() =>
    readPref<string[]>("mixerCollapsed", [], Array.isArray),
  );
  const toggle = (id: string) => {
    const next = toggleIds(collapsed, [id]);
    setCollapsed(next);
    writePref("mixerCollapsed", next);
  };
  return { collapsed, toggle };
}

function MixStatus({ film }: { film: Film }) {
  const unavailable = useMedia((s) => s.unavailable);
  const mixError = useMedia((s) => s.mixError);
  if (unavailable)
    return (
      <span className="text-muted-foreground">
        Set up the engine in Settings to hear the full mix and see levels.
      </span>
    );
  if (film.failed && mixError) return <span className="text-destructive truncate">{mixError}</span>;
  if (film.stale)
    return (
      <span className="text-muted-foreground flex items-center gap-1">
        <Spinner className="size-3" />
        Updating mix
      </span>
    );
  return <span className="text-muted-foreground">Levels show what the export plays.</span>;
}

function MixerToolbar({ film }: { film: Film }) {
  const setMix = useWorkflowStore((s) => s.setMix);
  return (
    <div className="flex h-8 shrink-0 items-center gap-2 border-b px-2 text-xs">
      <MixStatus film={film} />
      {soloActive(film.mix) ? <Badge variant="outline">Solo on</Badge> : null}
      <div className="flex-1" />
      <Button variant="ghost" size="sm" onClick={() => setMix((m) => addBus(m, randomId("bus")))}>
        <Plus data-icon="inline-start" />
        Add bus
      </Button>
    </div>
  );
}

export function MixerPanel() {
  const film = useFilm();
  const { mix, result } = film;
  const { collapsed, toggle } = useCollapsedBuses();

  const channels = film.plan?.channels ?? [];
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

  const holdFrames = Math.round(film.flat.fps * HOLD_SECONDS);
  const order = channels.map((c) => c.id);
  return (
    <div className="flex h-full flex-col">
      <MixerToolbar film={film} />
      <div className="flex min-h-0 flex-1 overflow-x-auto overflow-y-auto">
        <div className="flex min-h-full gap-px">
          {mix.buses.map((b) => {
            const members = channels.filter((c) => c.bus === b.id);
            const folded = collapsed.includes(b.id);
            return (
              <div key={b.id} className="flex gap-px border-r pr-px">
                {folded
                  ? null
                  : members.map((c) => (
                      <ChannelStrip
                        key={c.id}
                        channel={c}
                        order={order}
                        mix={mix}
                        result={result}
                        holdFrames={holdFrames}
                        sourceNodeId={film.sourceNodeId}
                      />
                    ))}
                <BusStrip
                  bus={b}
                  mix={mix}
                  members={members.map((c) => c.id)}
                  folded={folded}
                  onFold={() => toggle(b.id)}
                  result={result}
                  holdFrames={holdFrames}
                />
              </div>
            );
          })}
        </div>
        <div className="flex-1" />
        <MasterStrip master={mix.master} result={result} holdFrames={holdFrames} />
      </div>
    </div>
  );
}
