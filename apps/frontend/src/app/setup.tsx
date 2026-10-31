import { wholePercent } from "@opsis/video/common/format";
import { useNavigate } from "@tanstack/react-router";
import { CheckCircle2, Download, FileSearch } from "lucide-react";
import { useEffect, useState } from "react";
import { on } from "@/app/tauri";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Spinner } from "@/components/ui/spinner";
import { commands, events, type EngineInstallEvent, type ToolStatus } from "@/generated/bindings";
import { errorMessage } from "@/lib/errors";
import { useEnvironment, useFreshEnvironment } from "../store/environment";
import { PageBar } from "./app-bar";
import { AnimatedPercent } from "./motion";
import { SettingsCard } from "./settings/settings-card";
import { skipSetup } from "./startup-flags";

function useEngineInstall() {
  const [event, setEvent] = useState<EngineInstallEvent | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => on(events.engineInstallProgress, setEvent), []);

  async function install() {
    setBusy(true);
    setFailure(null);
    try {
      await commands.installEngine();
    } catch (e) {
      setFailure(errorMessage(e));
    } finally {
      setBusy(false);
      void useEnvironment.getState().refresh();
    }
  }

  return { event, error: event?.error ?? failure, busy, install };
}

function EngineProgress({
  event,
  error,
}: {
  event: EngineInstallEvent | null;
  error: string | null;
}) {
  return (
    <>
      {!error && event?.progress != null ? (
        <div key={event.step} className="flex items-center gap-2">
          <Progress className="flex-1" value={wholePercent(event.progress)} />
          <AnimatedPercent
            className="text-muted-foreground w-10 text-right text-xs"
            fraction={event.progress}
          />
        </div>
      ) : null}
      <p className={error ? "text-destructive text-xs" : "text-muted-foreground text-xs"}>
        {error ?? event?.message ?? "Starting…"}
      </p>
    </>
  );
}

function ToolState({ status }: { status: ToolStatus }) {
  if (status.checking)
    return (
      <span className="text-muted-foreground flex items-center gap-1.5 text-xs">
        <Spinner /> Checking…
      </span>
    );
  if (status.problem) return <p className="text-destructive text-xs">{status.problem}</p>;
  if (!status.path) return <p className="text-muted-foreground text-xs">Not set up</p>;
  return (
    <>
      <p className="text-muted-foreground text-xs">
        {status.version ? `Version ${status.version} · ` : null}
        <span className="font-mono break-all">{status.path}</span>
      </p>
      {status.warning ? <p className="text-warning text-xs">{status.warning}</p> : null}
    </>
  );
}

function BrowserRow({ status, disabled }: { status: ToolStatus; disabled: boolean }) {
  const [failure, setFailure] = useState<string | null>(null);
  const [choosing, setChoosing] = useState(false);
  async function choose() {
    setChoosing(true);
    setFailure(null);
    try {
      await commands.chooseBrowser();
    } catch (e) {
      setFailure(errorMessage(e));
    } finally {
      setChoosing(false);
    }
  }
  return (
    <div className="flex items-start gap-3 border-t pt-2 first:border-t-0 first:pt-0">
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className="flex items-center gap-1.5 text-sm font-medium">
          Recording browser
          {status.path && !status.problem && !status.checking ? (
            <CheckCircle2 className="text-success size-3.5" />
          ) : null}
        </span>
        <ToolState status={status} />
        {failure ? <p className="text-destructive text-xs">{failure}</p> : null}
      </div>
      <Button
        size="sm"
        variant="outline"
        disabled={disabled || choosing || status.checking}
        onClick={() => void choose()}
      >
        {choosing ? <Spinner data-icon="inline-start" /> : <FileSearch data-icon="inline-start" />}
        Choose file…
      </Button>
    </div>
  );
}

export function EnginePanel({ onDone, title = "Engine" }: { onDone?: () => void; title?: string }) {
  const env = useFreshEnvironment();
  const { event, error, busy, install } = useEngineInstall();
  const ready = !!env?.engine.complete;

  return (
    <SettingsCard
      title={title}
      badges={
        ready ? (
          <Badge variant="secondary">
            <CheckCircle2 data-icon="inline-start" />
            Set up
          </Badge>
        ) : (
          <Badge variant="outline">Not set up</Badge>
        )
      }
      description="The recording browser Opsis uses to record takes and draw exports. Download it, or choose one already on this computer; Opsis checks it before using it. Editing and preview work without it."
      contentClassName="flex flex-col gap-2"
      footer={
        <>
          <Button
            size="sm"
            variant={ready ? "outline" : "default"}
            disabled={busy}
            onClick={() => void install()}
          >
            {busy ? <Spinner data-icon="inline-start" /> : <Download data-icon="inline-start" />}
            {ready ? "Download again" : "Download engine"}
          </Button>
          {onDone ? (
            <Button size="sm" variant="ghost" onClick={onDone}>
              {ready ? "Continue" : "Skip for now"}
            </Button>
          ) : null}
        </>
      }
    >
      {env ? <BrowserRow status={env.engine.browser} disabled={busy} /> : null}
      {busy || error ? <EngineProgress event={event} error={error} /> : null}
    </SettingsCard>
  );
}

export function Setup() {
  const navigate = useNavigate();
  return (
    <div className="flex h-screen flex-col">
      <PageBar title="Welcome" />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto flex min-h-full max-w-xl flex-col justify-center gap-6 px-6 py-10">
          <div className="flex flex-col gap-1">
            <h1 className="text-2xl font-semibold tracking-tight">Welcome to Opsis</h1>
            <p className="text-muted-foreground text-sm">
              Recording and exporting need a recording browser. Download it once, or choose your
              own.
            </p>
          </div>
          <EnginePanel
            onDone={() => {
              skipSetup();
              void navigate({ to: "/" });
            }}
          />
        </div>
      </div>
    </div>
  );
}
