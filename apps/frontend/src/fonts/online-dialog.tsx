import { plural } from "@opsis/video/common/format";
import { Download } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { commands, type FontFamilyDef, type FontCatalogEntry } from "@/generated/bindings";
import { errorMessage } from "@/lib/errors";
import { useBusy } from "@/lib/use-busy";
import { categoryLabel } from "./font-labels";
import { FontRow } from "./font-row";
import { matchesQuery, SearchListDialog } from "./search-list-dialog";

type Catalog =
  | { status: "loading" }
  | { status: "error"; error: string }
  | { status: "ready"; entries: FontCatalogEntry[] };

function useCatalog(): { catalog: Catalog; retry: () => void; reload: () => void } {
  const [catalog, setCatalog] = useState<Catalog>({ status: "loading" });
  const fetchCatalog = useCallback(() => {
    commands.fontCatalog().then(
      (entries) => setCatalog({ status: "ready", entries }),
      (e: unknown) => setCatalog({ status: "error", error: errorMessage(e) }),
    );
  }, []);
  useEffect(fetchCatalog, [fetchCatalog]);
  function retry() {
    setCatalog({ status: "loading" });
    fetchCatalog();
  }
  return { catalog, retry, reload: fetchCatalog };
}

function CatalogStatus({ catalog, onRetry }: { catalog: Catalog; onRetry: () => void }) {
  if (catalog.status === "loading")
    return (
      <div className="text-muted-foreground flex items-center gap-2 p-2 text-sm">
        <Spinner /> Loading…
      </div>
    );
  if (catalog.status === "error")
    return (
      <div className="flex flex-col items-start gap-2 p-2">
        <p className="text-destructive text-sm">{catalog.error}</p>
        <Button size="xs" variant="outline" onClick={onRetry}>
          Try again
        </Button>
      </div>
    );
  return null;
}

function catalogDescription(catalog: Catalog): string {
  if (catalog.status === "ready")
    return `${catalog.entries.length} free fonts you can use in your videos.`;
  return catalog.status === "error" ? "The font list didn't load." : "Loading the font list…";
}

function DownloadIcon({ busy, added }: { busy: boolean; added: boolean }) {
  if (busy) return <Spinner data-icon="inline-start" />;
  return added ? null : <Download data-icon="inline-start" />;
}

function CatalogRow({
  entry,
  added,
  busy,
  disabled,
  onDownload,
}: {
  entry: FontCatalogEntry;
  added: boolean;
  busy: boolean;
  disabled: boolean;
  onDownload: () => void;
}) {
  return (
    <FontRow>
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="truncate font-medium">{entry.family}</span>
        <span className="text-muted-foreground truncate text-xs">
          {categoryLabel(entry.category)} · {plural(entry.weights.length, "weight")}
          {entry.styles.includes("italic") ? " · italics" : ""}
        </span>
      </span>
      <Button
        size="xs"
        variant={added ? "ghost" : "outline"}
        disabled={added || disabled}
        onClick={onDownload}
      >
        <DownloadIcon busy={busy} added={added} />
        {added ? "Added" : "Download"}
      </Button>
    </FontRow>
  );
}

function announceDownload(def: FontFamilyDef): void {
  if (def.licenseFile) toast.success(`Added ${def.family}`);
  else
    toast.warning(`Added ${def.family}, but its licence file didn't download`, {
      description: "Check that you're allowed to use this font in videos.",
    });
}

export function OnlineDialog({
  onClose,
  onAdded,
}: {
  onClose: () => void;
  onAdded: () => Promise<void>;
}) {
  const { catalog, retry, reload } = useCatalog();
  const [query, setQuery] = useState("");
  const { busy, isBusy, run } = useBusy();
  const entries = catalog.status === "ready" ? catalog.entries : null;
  const shown = (entries ?? []).filter((f) =>
    matchesQuery(query, f.family, categoryLabel(f.category)),
  );

  function download(entry: FontCatalogEntry) {
    return run(entry.id, async () => {
      announceDownload(await commands.downloadFont(entry.id));
      await onAdded();
      reload();
    });
  }

  return (
    <SearchListDialog
      onClose={onClose}
      title="Get fonts online"
      description={catalogDescription(catalog)}
      placeholder="Search by name or style (serif, monospace…)"
      query={query}
      onQuery={setQuery}
      items={shown}
      status={<CatalogStatus catalog={catalog} onRetry={retry} />}
      footer={
        <Button variant="outline" onClick={onClose}>
          Done
        </Button>
      }
    >
      {(f) => (
        <CatalogRow
          key={f.id}
          entry={f}
          added={f.added}
          busy={busy === f.id}
          disabled={isBusy}
          onDownload={() => void download(f)}
        />
      )}
    </SearchListDialog>
  );
}
