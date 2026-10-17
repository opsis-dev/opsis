import { Download } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import { familyKey } from "@/shared/fonts";
import { BUNDLED_FONTS } from "@/shared/fonts/bundled.generated";
import type { FontCatalogEntry } from "@/shared/ipc";
import { errorMessage } from "@/shared/errors";
import { plural } from "@/shared/format";
import { Button } from "@/renderer/components/ui/button";
import { Spinner } from "@/renderer/components/ui/spinner";
import { categoryLabel } from "./font-labels";
import { useFontLibrary } from "./library";
import { MAX_LISTED, SearchListDialog } from "./search-list-dialog";

type Catalog = { entries: FontCatalogEntry[] } | { error: string } | null;

function useCatalog(open: boolean): Catalog {
  const [catalog, setCatalog] = useState<Catalog>(null);
  const loaded = catalog !== null && "entries" in catalog;
  useEffect(() => {
    if (!open || loaded) return;
    window.demoflow.fontCatalog().then(
      (entries) => setCatalog({ entries }),
      (e: unknown) => setCatalog({ error: errorMessage(e) }),
    );
  }, [open, loaded]);
  return catalog;
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
        {busy ? (
          <Spinner data-icon="inline-start" />
        ) : added ? null : (
          <Download data-icon="inline-start" />
        )}
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
  onAdded: () => void;
}) {
  const catalog = useCatalog(open);
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [done, setDone] = useState<string[]>([]);
  const library = useFontLibrary((s) => s.library);

  const have = useMemo(
    () => new Set([...[...library, ...BUNDLED_FONTS].map((f) => familyKey(f.family)), ...done]),
    [library, done],
  );
  const entries = catalog && "entries" in catalog ? catalog.entries : null;
  const q = query.trim().toLowerCase();
  const shown = (entries ?? []).filter(
    (f) =>
      !q || familyKey(f.family).includes(q) || categoryLabel(f.category).toLowerCase().includes(q),
  );

  const download = async (entry: FontCatalogEntry) => {
    setBusy(entry.id);
    try {
      const def = await window.demoflow.downloadFont(entry.id);
      setDone((d) => [...d, familyKey(def.family)]);
      toast.success(`Added ${def.family}`);
      onAdded();
    } catch (e) {
      toast.error(errorMessage(e));
    } finally {
      setBusy(null);
    }
  };

  const status =
    catalog === null ? (
      <div className="text-muted-foreground flex items-center gap-2 p-2 text-sm">
        <Spinner /> Loading…
      </div>
    ) : "error" in catalog ? (
      <p className="text-destructive p-2 text-sm">{catalog.error}</p>
    ) : null;

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
      matches={shown.length}
      status={status}
      footer={
        <Button variant="outline" onClick={onClose}>
          Done
        </Button>
      }
    >
      {shown.slice(0, MAX_LISTED).map((f) => (
        <CatalogRow
          key={f.id}
          entry={f}
          added={have.has(familyKey(f.family))}
          busy={busy === f.id}
          disabled={!!busy}
          onDownload={() => void download(f)}
        />
      ))}
    </SearchListDialog>
  );
}
