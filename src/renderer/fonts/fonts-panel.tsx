import { Download, FileArchive, Globe, HardDrive, Trash2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import { BUNDLED_FONTS, type FontFamilyDef } from "@/shared/fonts";
import type { FontCatalogEntry, FontInstalledFace, InspectedFonts } from "@/shared/ipc";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/renderer/components/ui/alert-dialog";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/renderer/components/ui/card";
import { Checkbox } from "@/renderer/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/renderer/components/ui/dialog";
import { Input } from "@/renderer/components/ui/input";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { Spinner } from "@/renderer/components/ui/spinner";
import { registerPreview, useFontLibrary } from "./library";

const SOURCE_LABEL: Record<FontFamilyDef["source"], string> = {
  bundled: "Built in",
  online: "Downloaded",
  file: "From files",
  installed: "From this computer",
};

const CATEGORY_LABEL: Record<string, string> = {
  "sans-serif": "Sans serif",
  serif: "Serif",
  monospace: "Monospace",
  display: "Display",
  handwriting: "Handwriting",
  icons: "Icons",
  other: "Other",
};

const WEIGHT_NAMES: Record<string, string> = {
  "100": "Thin",
  "200": "Extra light",
  "300": "Light",
  "400": "Regular",
  "500": "Medium",
  "600": "Semibold",
  "700": "Bold",
  "800": "Extra bold",
  "900": "Black",
};

export function describeFaces(faces: Array<{ weight: string; style: string }>): string {
  const parts = faces.map((f) => {
    const name = f.weight.includes(" ")
      ? `All weights (${f.weight.replace(" ", "–")})`
      : (WEIGHT_NAMES[f.weight] ?? f.weight);
    return f.style === "italic" ? `${name} italic` : name;
  });
  return parts.join(", ");
}

function Sample({ def }: { def: FontFamilyDef }) {
  const family = registerPreview(def);
  return (
    <span className="truncate text-base" style={{ fontFamily: `"${family}", sans-serif` }}>
      {def.family}
    </span>
  );
}

type LocalFontData = {
  family: string;
  style: string;
  postscriptName: string;
  blob: () => Promise<Blob>;
};

async function installedFonts(): Promise<LocalFontData[]> {
  const query = (window as unknown as { queryLocalFonts?: () => Promise<LocalFontData[]> })
    .queryLocalFonts;
  if (!query) throw new Error("This computer doesn't let apps list installed fonts.");
  return query();
}

let onlineAllowed = false;

export function FontsPanel() {
  const { library, refresh } = useFontLibrary();
  const [review, setReview] = useState<{ fonts: InspectedFonts; installed: boolean } | null>(null);
  const [installed, setInstalled] = useState<LocalFontData[] | null>(null);
  const [askOnline, setAskOnline] = useState(false);
  const [online, setOnline] = useState(false);
  const [removing, setRemoving] = useState<FontFamilyDef | null>(null);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const addFiles = async () => {
    try {
      const fonts = await window.demoflow.inspectFontFiles();
      if (!fonts) return;
      if (!fonts.families.length) return void toast.error("No fonts found in those files.");
      setReview({ fonts, installed: false });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  };

  const openInstalled = async () => {
    try {
      setInstalled(await installedFonts());
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Your library</CardTitle>
        <CardDescription>
          {BUNDLED_FONTS.length} fonts come with Demoflow. Fonts you add here work in every project;
          a project keeps its own copy of the fonts it uses.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-1">
        {library.length === 0 ? (
          <p className="text-muted-foreground text-sm">No fonts added yet.</p>
        ) : (
          library.map((def) => (
            <div key={def.id} className="flex items-center gap-3 rounded-md px-2 py-1.5">
              <div className="flex min-w-0 flex-1 flex-col">
                <Sample def={def} />
                <span className="text-muted-foreground truncate text-xs">
                  {describeFaces(def.faces)}
                </span>
              </div>
              <Badge variant="secondary">{SOURCE_LABEL[def.source]}</Badge>
              {def.licenseFile ? <Badge variant="outline">Licence included</Badge> : null}
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label={`Remove ${def.family}`}
                onClick={() => setRemoving(def)}
              >
                <Trash2 />
              </Button>
            </div>
          ))
        )}
      </CardContent>
      <CardFooter className="flex-wrap gap-2">
        <Button variant="outline" size="sm" onClick={() => void addFiles()}>
          <FileArchive data-icon="inline-start" />
          Add font files or ZIP…
        </Button>
        <Button variant="outline" size="sm" onClick={() => void openInstalled()}>
          <HardDrive data-icon="inline-start" />
          Add installed fonts…
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={() => (onlineAllowed ? setOnline(true) : setAskOnline(true))}
        >
          <Globe data-icon="inline-start" />
          Get fonts online…
        </Button>
      </CardFooter>

      <ReviewDialog
        key={review?.fonts.token ?? "none"}
        value={review}
        onClose={() => setReview(null)}
        onAdded={() => void refresh()}
      />
      <InstalledDialog
        fonts={installed}
        onClose={() => setInstalled(null)}
        onPicked={(fonts) => {
          setInstalled(null);
          setReview({ fonts, installed: true });
        }}
      />
      <AlertDialog open={askOnline} onOpenChange={setAskOnline}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Go online to get fonts?</AlertDialogTitle>
            <AlertDialogDescription>
              Demoflow will connect to the internet to show the list of free fonts, and to download
              the ones you pick. Each font is saved to your library and works offline afterwards.
              Nothing about you or your projects is sent.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                onlineAllowed = true;
                setOnline(true);
              }}
            >
              Connect
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
      <OnlineDialog open={online} onClose={() => setOnline(false)} onAdded={() => void refresh()} />
      <AlertDialog open={!!removing} onOpenChange={(o) => !o && setRemoving(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Remove {removing?.family}?</AlertDialogTitle>
            <AlertDialogDescription>
              It's removed from your library. Projects that already use it keep their own copy.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => {
                if (!removing) return;
                void window.demoflow.removeFont(removing.id).then(() => refresh());
              }}
            >
              Remove
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </Card>
  );
}

function ReviewDialog({
  value,
  onClose,
  onAdded,
}: {
  value: { fonts: InspectedFonts; installed: boolean } | null;
  onClose: () => void;
  onAdded: () => void;
}) {
  const [chosen, setChosen] = useState<string[]>(
    () => value?.fonts.families.map((f) => f.family) ?? [],
  );
  const [busy, setBusy] = useState(false);
  if (!value) return null;
  const { fonts, installed } = value;
  const add = async () => {
    setBusy(true);
    try {
      const added = await window.demoflow.addInspectedFonts(fonts.token, chosen);
      toast.success(
        added.length === 1 ? `Added ${added[0].family}` : `Added ${added.length} fonts`,
      );
      onAdded();
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog open onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Add fonts</DialogTitle>
          <DialogDescription>
            {fonts.families.length === 1
              ? "Found 1 font family."
              : `Found ${fonts.families.length} font families.`}{" "}
            {fonts.skipped ? `${fonts.skipped} duplicate files in other formats were skipped.` : ""}
          </DialogDescription>
        </DialogHeader>
        <ScrollArea className="max-h-72">
          <div className="flex flex-col gap-2 pr-3">
            {fonts.families.map((f) => (
              <label key={f.family} className="flex items-start gap-3 text-sm">
                <Checkbox
                  checked={chosen.includes(f.family)}
                  onCheckedChange={(on) =>
                    setChosen((c) => (on ? [...c, f.family] : c.filter((x) => x !== f.family)))
                  }
                />
                <span className="flex min-w-0 flex-col">
                  <span className="font-medium">{f.family}</span>
                  <span className="text-muted-foreground text-xs">{describeFaces(f.faces)}</span>
                </span>
              </label>
            ))}
          </div>
        </ScrollArea>
        <p className="text-muted-foreground text-xs">
          {fonts.license
            ? `Licence file found: ${fonts.license}. It's kept with the fonts.`
            : installed
              ? "Fonts installed on a computer often have licences that limit use in videos. Check yours before you publish."
              : "No licence file found. Check that you're allowed to use these fonts in videos."}
        </p>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button disabled={!chosen.length || busy} onClick={() => void add()}>
            {busy ? <Spinner data-icon="inline-start" /> : null}
            Add {chosen.length === 1 ? "1 font" : `${chosen.length} fonts`}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function InstalledDialog({
  fonts,
  onClose,
  onPicked,
}: {
  fonts: LocalFontData[] | null;
  onClose: () => void;
  onPicked: (fonts: InspectedFonts) => void;
}) {
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const families = useMemo(() => {
    const map = new Map<string, LocalFontData[]>();
    for (const f of fonts ?? []) map.set(f.family, [...(map.get(f.family) ?? []), f]);
    return [...map.entries()].toSorted(([a], [b]) => a.localeCompare(b));
  }, [fonts]);
  if (!fonts) return null;
  const shown = families.filter(([name]) => name.toLowerCase().includes(query.toLowerCase()));
  const pick = async (family: string, faces: LocalFontData[]) => {
    setBusy(family);
    try {
      const data: FontInstalledFace[] = [];
      for (const face of faces) {
        const buffer = await (await face.blob()).arrayBuffer();
        const head = new Uint8Array(buffer, 0, 4);
        const otto = String.fromCharCode(...head) === "OTTO";
        data.push({
          postscriptName: face.postscriptName,
          format: otto ? "opentype" : "truetype",
          data: buffer,
        });
      }
      onPicked(await window.demoflow.inspectInstalledFonts(data));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(null);
    }
  };
  return (
    <Dialog open onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Add an installed font</DialogTitle>
          <DialogDescription>
            {families.length} font families are installed on this computer. Pick one to copy into
            your library.
          </DialogDescription>
        </DialogHeader>
        <Input
          placeholder="Search installed fonts…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          autoFocus
        />
        <ScrollArea className="h-72">
          <div className="flex flex-col pr-3">
            {shown.slice(0, 200).map(([name, faces]) => (
              <button
                key={name}
                type="button"
                className="hover:bg-muted flex items-center justify-between rounded-md px-2 py-1.5 text-left text-sm"
                disabled={!!busy}
                onClick={() => void pick(name, faces)}
              >
                <span className="truncate" style={{ fontFamily: `"${name}"` }}>
                  {name}
                </span>
                <span className="text-muted-foreground flex items-center gap-2 text-xs">
                  {busy === name ? <Spinner /> : null}
                  {faces.length === 1 ? "1 style" : `${faces.length} styles`}
                </span>
              </button>
            ))}
          </div>
        </ScrollArea>
      </DialogContent>
    </Dialog>
  );
}

function OnlineDialog({
  open,
  onClose,
  onAdded,
}: {
  open: boolean;
  onClose: () => void;
  onAdded: () => void;
}) {
  const [catalog, setCatalog] = useState<FontCatalogEntry[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [done, setDone] = useState<string[]>([]);
  const library = useFontLibrary((s) => s.library);

  useEffect(() => {
    if (!open || catalog) return;
    window.demoflow
      .fontCatalog()
      .then(setCatalog, (e: unknown) => setError(e instanceof Error ? e.message : String(e)));
  }, [open, catalog]);

  const have = new Set(
    [...library, ...BUNDLED_FONTS].map((f) => f.family.toLowerCase()).concat(done),
  );
  const q = query.trim().toLowerCase();
  const shown = (catalog ?? [])
    .filter(
      (f) =>
        !q ||
        f.family.toLowerCase().includes(q) ||
        (CATEGORY_LABEL[f.category] ?? f.category).toLowerCase().includes(q),
    )
    .slice(0, 100);

  const download = async (entry: FontCatalogEntry) => {
    setBusy(entry.id);
    try {
      const def = await window.demoflow.downloadFont(entry.id);
      setDone((d) => [...d, def.family.toLowerCase()]);
      toast.success(`Added ${def.family}`);
      onAdded();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(null);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Get fonts online</DialogTitle>
          <DialogDescription>
            {catalog
              ? `${catalog.length} free fonts you can use in your videos.`
              : "Loading the font list…"}
          </DialogDescription>
        </DialogHeader>
        <Input
          placeholder="Search by name or style (serif, monospace…)"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          autoFocus
        />
        {error ? <p className="text-destructive text-sm">{error}</p> : null}
        <ScrollArea className="h-80">
          <div className="flex flex-col pr-3">
            {!catalog && !error ? (
              <div className="text-muted-foreground flex items-center gap-2 p-2 text-sm">
                <Spinner /> Loading…
              </div>
            ) : null}
            {shown.map((f) => {
              const added = have.has(f.family.toLowerCase());
              return (
                <div key={f.id} className="flex items-center gap-3 rounded-md px-2 py-1.5 text-sm">
                  <span className="flex min-w-0 flex-1 flex-col">
                    <span className="truncate font-medium">{f.family}</span>
                    <span className="text-muted-foreground truncate text-xs">
                      {CATEGORY_LABEL[f.category] ?? "Other"} ·{" "}
                      {f.weights.length === 1 ? "1 weight" : `${f.weights.length} weights`}
                      {f.styles.includes("italic") ? " · italics" : ""}
                    </span>
                  </span>
                  <Button
                    size="xs"
                    variant={added ? "ghost" : "outline"}
                    disabled={added || !!busy}
                    onClick={() => void download(f)}
                  >
                    {busy === f.id ? (
                      <Spinner data-icon="inline-start" />
                    ) : added ? null : (
                      <Download data-icon="inline-start" />
                    )}
                    {added ? "Added" : "Download"}
                  </Button>
                </div>
              );
            })}
            {catalog && shown.length === 100 ? (
              <p className="text-muted-foreground px-2 py-1 text-xs">
                Showing the first 100 matches. Type to narrow the list.
              </p>
            ) : null}
          </div>
        </ScrollArea>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Done
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
