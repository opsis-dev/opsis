import { Download } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import { familyKey, familyKeysWithBundled } from "@/shared/fonts";
import type { FontCatalogEntry } from "@/shared/ipc";
import { errorMessage } from "@/shared/errors";
import { plural } from "@/shared/format";
import { Button } from "@/renderer/components/ui/button";
import { Spinner } from "@/renderer/components/ui/spinner";
import { categoryLabel } from "./font-labels";
import { useFontLibrary } from "./library";
import { useBusy } from "@/renderer/lib/use-busy";
import { matchesQuery, SearchListDialog } from "./search-list-dialog";

type Catalog =
  | { status: "loading" }
  | { status: "error"; error: string }
  | { status: "ready"; entries: FontCatalogEntry[] };

function fetchCatalog(show: (catalog: Catalog) => void): void {
  window.demoflow.fontCatalog().then(
    (entries) => show({ status: "ready", entries }),
    (e: unknown) => show({ status: "error", error: errorMessage(e) }),
  );
}

function useCatalog(open: boolean): { catalog: Catalog; retry: () => void } {
  const [catalog, setCatalog] = useState<Catalog>({ status: "loading" });
  const ready = catalog.status === "ready";
  useEffect(() => {
    if (!open || ready) return;
    let current = true;
    fetchCatalog((next) => current && setCatalog(next));
    return () => {
      current = false;
    };
  }, [open, ready]);
  const retry = () => {
    setCatalog({ status: "loading" });
    fetchCatalog(setCatalog);
  };
  return { catalog, retry };
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
    <div className="flex items-center gap-3 rounded-md px-2 py-1.5 text-sm">
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
    </div>
  );
}

export function OnlineDialog({
  open,
  onClose,
  onAdded,
}: {
  open: boolean;
  onClose: () => void;
  onAdded: () => Promise<void>;
}) {
  const { catalog, retry } = useCatalog(open);
  const [query, setQuery] = useState("");
  const { busy, isBusy, run } = useBusy();
  const library = useFontLibrary((s) => s.library);

  const have = useMemo(() => familyKeysWithBundled(library), [library]);
  const entries = catalog.status === "ready" ? catalog.entries : null;
  const shown = (entries ?? []).filter((f) =>
    matchesQuery(query, f.family, categoryLabel(f.category)),
  );

  const download = (entry: FontCatalogEntry) =>
    run(entry.id, async () => {
      const def = await window.demoflow.downloadFont(entry.id);
      toast.success(`Added ${def.family}`);
      await onAdded();
    });

  return (
    <SearchListDialog
      open={open}
      onClose={onClose}
      title="Get fonts online"
      description={
        entries
          ? `${entries.length} free fonts you can use in your videos.`
          : "Loading the font list…"
      }
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
          added={have.has(familyKey(f.family))}
          busy={busy === f.id}
          disabled={isBusy}
          onDownload={() => void download(f)}
        />
      )}
    </SearchListDialog>
  );
}
