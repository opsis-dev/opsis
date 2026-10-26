import { ChevronRight, ExternalLink } from "lucide-react";
import { useEffect, useState, type ReactNode } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { Spinner } from "@/components/ui/spinner";
import { errorMessage } from "@/core/errors";
import type { FontFamilyDef, LicenceEntry } from "@/generated/bindings";
import { attempt } from "@/lib/toast";
import { bundledFonts } from "@/store/catalog";
import { fileUrl } from "../../editor/file-url";
import { useFontLibrary } from "../../fonts/library";
import { useLoaded } from "../use-loaded";
import { SettingsCard } from "./settings-card";

function LicenceText({ text }: { text: string }) {
  return (
    <pre className="bg-muted text-muted-foreground max-h-96 overflow-auto rounded-md p-3 font-mono text-xs leading-5 whitespace-pre-wrap">
      {text}
    </pre>
  );
}

async function readFontLicence(def: FontFamilyDef): Promise<string> {
  if (!def.licenseFile) return "No licence file came with this font.";
  const res = await fetch(fileUrl(def.licenseFile));
  if (!res.ok) throw new Error(`Error ${res.status}.`);
  return res.text();
}

function FontLicenceText({ def }: { def: FontFamilyDef }) {
  const [read, setRead] = useState<{ def: FontFamilyDef; text: string } | null>(null);
  useEffect(() => {
    let current = true;
    const show = (text: string) => current && setRead({ def, text });
    readFontLicence(def).then(show, (e: unknown) =>
      show(`This licence file couldn't be read. ${errorMessage(e)}`),
    );
    return () => {
      current = false;
    };
  }, [def]);
  return read?.def === def ? <LicenceText text={read.text} /> : <Spinner />;
}

type RowProps = { title: string; summary: string; badge?: string };

function RowHead({ title, summary, badge }: RowProps) {
  return (
    <div className="flex min-w-0 flex-1 flex-col items-start gap-0.5 text-left">
      <span className="flex items-center gap-2 text-sm font-medium">
        {title}
        {badge ? <Badge variant="outline">{badge}</Badge> : null}
      </span>
      <span className="text-muted-foreground text-xs">{summary}</span>
    </div>
  );
}

function FileRow({ file, ...head }: RowProps & { file: string }) {
  return (
    <div className="flex items-center gap-3 py-2">
      <RowHead {...head} />
      <Button
        size="sm"
        variant="outline"
        onClick={() =>
          void attempt(() => window.opsis.openLicenceFile(file), "Couldn't open the licence file")
        }
      >
        <ExternalLink data-icon="inline-start" />
        Open
      </Button>
    </div>
  );
}

function Row({ children, ...head }: RowProps & { children?: ReactNode }) {
  return (
    <Collapsible className="py-2">
      <CollapsibleTrigger className="group/licence flex w-full items-center gap-3">
        <RowHead {...head} />
        <ChevronRight className="text-muted-foreground size-4 shrink-0 group-data-[panel-open]/licence:rotate-90" />
      </CollapsibleTrigger>
      <CollapsibleContent className="pt-2">{children}</CollapsibleContent>
    </Collapsible>
  );
}

function SoftwareRow({ entry }: { entry: LicenceEntry }) {
  if (entry.file) return <FileRow title={entry.title} summary={entry.summary} file={entry.file} />;
  return (
    <Row title={entry.title} summary={entry.summary}>
      {entry.text ? <LicenceText text={entry.text} /> : null}
    </Row>
  );
}

function FontLicenceRows({ fonts }: { fonts: FontFamilyDef[] }) {
  return fonts.map((f) => (
    <Row
      key={f.id}
      title={f.family}
      summary={f.license ?? "Licence not listed"}
      badge={f.source === "online" ? "Downloaded" : undefined}
    >
      <FontLicenceText def={f} />
    </Row>
  ));
}

export function LicencesPage() {
  const entries = useLoaded(() => window.opsis.listLicences(), "Couldn't list the licences");
  const library = useFontLibrary((s) => s.library);
  useEffect(() => {
    void useFontLibrary.getState().refresh();
  }, []);

  return (
    <>
      <SettingsCard
        title="Software"
        description="Opsis and the programs it ships with or sets up. The recording browser and the video tools appear once the engine is set up."
        contentClassName="divide-y"
      >
        {entries === null ? <Spinner /> : entries.map((e) => <SoftwareRow key={e.id} entry={e} />)}
      </SettingsCard>
      <SettingsCard
        title="Built-in fonts"
        description="Fonts that come with Opsis."
        contentClassName="divide-y"
      >
        <FontLicenceRows fonts={bundledFonts()} />
      </SettingsCard>
      {library.length ? (
        <SettingsCard
          title="Your fonts"
          description="Fonts in your library, with the licence they came with."
          contentClassName="divide-y"
        >
          <FontLicenceRows fonts={library.map((choice) => choice.def)} />
        </SettingsCard>
      ) : null}
    </>
  );
}
