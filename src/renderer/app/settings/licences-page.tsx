import { ChevronRight, ExternalLink } from "lucide-react";
import { useEffect, useState, type ReactNode } from "react";
import { toast } from "sonner";
import { BUNDLED_FONTS, type FontFamilyDef } from "@/shared/fonts";
import type { LicenceEntry } from "@/shared/ipc";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/renderer/components/ui/card";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/renderer/components/ui/collapsible";
import { Spinner } from "@/renderer/components/ui/spinner";
import { fontFileUrl } from "../../fonts/library";
import { errorMessage } from "@/shared/errors";
import { useLoaded } from "../use-loaded";

function LicenceText({ text }: { text: string }) {
  return (
    <pre className="bg-muted text-muted-foreground max-h-96 overflow-auto rounded-md p-3 font-mono text-xs leading-5 whitespace-pre-wrap">
      {text}
    </pre>
  );
}

async function readFontLicence(def: FontFamilyDef): Promise<string> {
  if (!def.licenseFile) return "No licence file came with this font.";
  const res = await fetch(fontFileUrl(def.licenseFile));
  if (!res.ok) throw new Error(String(res.status));
  return res.text();
}

function FontLicenceText({ def }: { def: FontFamilyDef }) {
  const [text, setText] = useState<string | null>(null);
  useEffect(() => {
    readFontLicence(def).then(setText, () => setText("This licence file couldn't be read."));
  }, [def]);
  return text === null ? <Spinner /> : <LicenceText text={text} />;
}

function RowHead({ title, summary, badge }: { title: string; summary: string; badge?: string }) {
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

type RowProps = { title: string; summary: string; badge?: string };

function FileRow({ file, ...head }: RowProps & { file: string }) {
  return (
    <div className="flex items-center gap-3 py-2">
      <RowHead {...head} />
      <Button
        size="sm"
        variant="outline"
        onClick={() =>
          void window.demoflow.openLicenceFile(file).catch((e: unknown) =>
            toast.error("Couldn't open the licence file", {
              description: errorMessage(e),
            }),
          )
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
  const entries = useLoaded(() => window.demoflow.listLicences(), "Could not list the licences");
  const library = useLoaded(() => window.demoflow.listFonts(), "Could not read your font library");

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>Software</CardTitle>
          <CardDescription>
            Demoflow and the programs it ships with or sets up. The recording browser and the video
            tools appear once the engine is set up.
          </CardDescription>
        </CardHeader>
        <CardContent className="divide-y">
          {entries === null ? (
            <Spinner />
          ) : (
            entries.map((e) => <SoftwareRow key={e.id} entry={e} />)
          )}
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>Built-in fonts</CardTitle>
          <CardDescription>Fonts that come with Demoflow.</CardDescription>
        </CardHeader>
        <CardContent className="divide-y">
          <FontLicenceRows fonts={BUNDLED_FONTS} />
        </CardContent>
      </Card>
      {library?.length ? (
        <Card>
          <CardHeader>
            <CardTitle>Your fonts</CardTitle>
            <CardDescription>
              Fonts in your library, with the licence they came with.
            </CardDescription>
          </CardHeader>
          <CardContent className="divide-y">
            <FontLicenceRows fonts={library} />
          </CardContent>
        </Card>
      ) : null}
    </>
  );
}
