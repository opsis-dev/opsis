import { ChevronRight, ExternalLink } from "lucide-react";
import { useEffect, useState } from "react";
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

function LicenceText({ load }: { load: () => Promise<string> }) {
  const [text, setText] = useState<string | null>(null);
  useEffect(() => {
    void load()
      .then(setText)
      .catch(() => setText("This licence file couldn't be read."));
  }, [load]);
  if (text === null) return <Spinner />;
  return (
    <pre className="bg-muted text-muted-foreground max-h-96 overflow-auto rounded-md p-3 font-mono text-xs leading-5 whitespace-pre-wrap">
      {text}
    </pre>
  );
}

function Row({
  title,
  summary,
  badge,
  load,
  file,
}: {
  title: string;
  summary: string;
  badge?: string;
  load?: () => Promise<string>;
  file?: string;
}) {
  const head = (
    <div className="flex min-w-0 flex-1 flex-col items-start gap-0.5 text-left">
      <span className="flex items-center gap-2 text-sm font-medium">
        {title}
        {badge ? <Badge variant="outline">{badge}</Badge> : null}
      </span>
      <span className="text-muted-foreground text-xs">{summary}</span>
    </div>
  );
  if (file)
    return (
      <div className="flex items-center gap-3 py-2">
        {head}
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
  return (
    <Collapsible className="py-2">
      <CollapsibleTrigger className="group/licence flex w-full items-center gap-3">
        {head}
        <ChevronRight className="text-muted-foreground size-4 shrink-0 group-data-[panel-open]/licence:rotate-90" />
      </CollapsibleTrigger>
      <CollapsibleContent className="pt-2">
        {load ? <LicenceText load={load} /> : null}
      </CollapsibleContent>
    </Collapsible>
  );
}

const fontLicence = (def: FontFamilyDef) => async () => {
  if (!def.licenseFile) return "No licence file came with this font.";
  const res = await fetch(fontFileUrl(def.licenseFile));
  if (!res.ok) throw new Error(String(res.status));
  return res.text();
};

export function LicencesPage() {
  const [entries, setEntries] = useState<LicenceEntry[] | null>(null);
  const [library, setLibrary] = useState<FontFamilyDef[]>([]);

  useEffect(() => {
    void window.demoflow.listLicences().then(setEntries);
    void window.demoflow.listFonts().then(setLibrary);
  }, []);

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
            entries.map((e) => (
              <Row
                key={e.id}
                title={e.title}
                summary={e.summary}
                file={e.file}
                load={e.text ? () => Promise.resolve(e.text!) : undefined}
              />
            ))
          )}
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>Built-in fonts</CardTitle>
          <CardDescription>Fonts that come with Demoflow.</CardDescription>
        </CardHeader>
        <CardContent className="divide-y">
          {BUNDLED_FONTS.map((f) => (
            <Row
              key={f.id}
              title={f.family}
              summary={f.license ?? "Licence not listed"}
              load={fontLicence(f)}
            />
          ))}
        </CardContent>
      </Card>
      {library.length ? (
        <Card>
          <CardHeader>
            <CardTitle>Your fonts</CardTitle>
            <CardDescription>
              Fonts in your library, with the licence they came with.
            </CardDescription>
          </CardHeader>
          <CardContent className="divide-y">
            {library.map((f) => (
              <Row
                key={f.id}
                title={f.family}
                summary={f.license ?? "Licence not listed"}
                badge={f.source === "online" ? "Downloaded" : undefined}
                load={fontLicence(f)}
              />
            ))}
          </CardContent>
        </Card>
      ) : null}
    </>
  );
}
