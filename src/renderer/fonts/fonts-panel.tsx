import { FileArchive, Globe, HardDrive, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { BUNDLED_FONTS } from "@/shared/fonts/bundled.generated";
import type { FontFamilyDef } from "@/shared/fonts/types";
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
import { attempt } from "@/renderer/lib/toast";
import { describeFaces } from "./font-labels";
import { InstalledDialog } from "./installed-dialog";
import { FontSample } from "./font-sample";
import { useFontLibrary } from "./library";
import { installedFonts, type LocalFontData } from "./local-fonts";
import { OnlineDialog } from "./online-dialog";
import { ReviewDialog, type FontReview } from "./review-dialog";

const SOURCE_LABEL: Record<FontFamilyDef["source"], string> = {
  bundled: "Built in",
  online: "Downloaded",
  file: "From files",
  installed: "From this computer",
};

const refreshLibrary = () => useFontLibrary.getState().refresh();

function LibraryRow({ def, onRemove }: { def: FontFamilyDef; onRemove: () => void }) {
  return (
    <div className="flex items-center gap-3 rounded-md px-2 py-1.5">
      <div className="flex min-w-0 flex-1 flex-col">
        <FontSample def={def} className="text-base">
          {def.family}
        </FontSample>
        <span className="text-muted-foreground truncate text-xs">{describeFaces(def.faces)}</span>
      </div>
      <Badge variant="secondary">{SOURCE_LABEL[def.source]}</Badge>
      {def.licenseFile ? <Badge variant="outline">Licence included</Badge> : null}
      <Button variant="ghost" size="icon-sm" aria-label={`Remove ${def.family}`} onClick={onRemove}>
        <Trash2 />
      </Button>
    </div>
  );
}

function OnlineConsentDialog({
  open,
  onOpenChange,
  onConnect,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConnect: () => void;
}) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
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
          <AlertDialogAction onClick={onConnect}>Connect</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

function RemoveFontDialog({ font, onClose }: { font: FontFamilyDef | null; onClose: () => void }) {
  const remove = async (def: FontFamilyDef) => {
    await attempt(async () => {
      await window.demoflow.removeFont(def.id);
      await refreshLibrary();
    }, `Could not remove ${def.family}`);
    onClose();
  };

  return (
    <AlertDialog open={!!font} onOpenChange={(o) => !o && onClose()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Remove {font?.family}?</AlertDialogTitle>
          <AlertDialogDescription>
            It's removed from your library. Projects that already use it keep their own copy.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction variant="destructive" onClick={() => font && void remove(font)}>
            Remove
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

type PanelDialog =
  | { kind: "review"; review: FontReview }
  | { kind: "installed"; fonts: LocalFontData[] }
  | { kind: "ask-online" }
  | { kind: "online" }
  | { kind: "remove"; font: FontFamilyDef };

export function FontsPanel() {
  const library = useFontLibrary((s) => s.library);
  const onlineAllowed = useFontLibrary((s) => s.onlineAllowed);
  const [dialog, setDialog] = useState<PanelDialog | null>(null);
  const closeIf = (kind: PanelDialog["kind"]) => () =>
    setDialog((d) => (d?.kind === kind ? null : d));

  useEffect(() => {
    void refreshLibrary();
  }, []);

  const addFiles = () =>
    attempt(async () => {
      const fonts = await window.demoflow.inspectFontFiles();
      if (!fonts) return;
      if (!fonts.families.length) return void toast.error("No fonts found in those files.");
      setDialog({ kind: "review", review: { fonts, installed: false } });
    });

  const openInstalled = () =>
    attempt(async () => setDialog({ kind: "installed", fonts: await installedFonts() }));

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
            <LibraryRow
              key={def.id}
              def={def}
              onRemove={() => setDialog({ kind: "remove", font: def })}
            />
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
          onClick={() => setDialog({ kind: onlineAllowed ? "online" : "ask-online" })}
        >
          <Globe data-icon="inline-start" />
          Get fonts online…
        </Button>
      </CardFooter>

      {dialog?.kind === "review" ? (
        <ReviewDialog
          review={dialog.review}
          onClose={closeIf("review")}
          onAdded={() => void refreshLibrary()}
        />
      ) : null}
      {dialog?.kind === "installed" ? (
        <InstalledDialog
          fonts={dialog.fonts}
          onClose={closeIf("installed")}
          onPicked={(fonts) => setDialog({ kind: "review", review: { fonts, installed: true } })}
        />
      ) : null}
      <OnlineConsentDialog
        open={dialog?.kind === "ask-online"}
        onOpenChange={(open) => !open && closeIf("ask-online")()}
        onConnect={() => {
          useFontLibrary.getState().allowOnline();
          setDialog({ kind: "online" });
        }}
      />
      <OnlineDialog
        open={dialog?.kind === "online"}
        onClose={closeIf("online")}
        onAdded={refreshLibrary}
      />
      <RemoveFontDialog
        font={dialog?.kind === "remove" ? dialog.font : null}
        onClose={closeIf("remove")}
      />
    </Card>
  );
}
