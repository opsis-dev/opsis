import { FileArchive, Globe, HardDrive, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  commands,
  type FontChoice,
  type FontFamilyDef,
  type InstalledFamily,
} from "@/generated/bindings";
import { attempt } from "@/lib/toast";
import { bundledFonts } from "@/store/catalog";
import { SettingsCard } from "../app/settings/settings-card";
import { describeFaces } from "./font-labels";
import { FontRow } from "./font-row";
import { FontSample } from "./font-sample";
import { InstalledDialog } from "./installed-dialog";
import { useFontLibrary } from "./library";
import { OnlineDialog } from "./online-dialog";
import { ReviewDialog, type FontReview } from "./review-dialog";

const SOURCE_LABEL: Record<FontFamilyDef["source"], string> = {
  bundled: "Built in",
  online: "Downloaded",
  file: "From files",
  installed: "From this computer",
};

const refreshLibrary = () => useFontLibrary.getState().refresh();

function LibraryRow({ choice, onRemove }: { choice: FontChoice; onRemove: () => void }) {
  const { def } = choice;
  return (
    <FontRow>
      <div className="flex min-w-0 flex-1 flex-col">
        <FontSample choice={choice} className="text-base">
          {def.family}
        </FontSample>
        <span className="text-muted-foreground truncate text-xs">{describeFaces(def.faces)}</span>
      </div>
      <Badge variant="secondary">{SOURCE_LABEL[def.source]}</Badge>
      {def.licenseFile ? <Badge variant="outline">Licence included</Badge> : null}
      <Button variant="ghost" size="icon-sm" aria-label={`Remove ${def.family}`} onClick={onRemove}>
        <Trash2 />
      </Button>
    </FontRow>
  );
}

let onlineAllowed = false;

function OnlineConsentDialog({
  onClose,
  onConnect,
}: {
  onClose: () => void;
  onConnect: () => void;
}) {
  return (
    <AlertDialog open onOpenChange={(o) => !o && onClose()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Go online to get fonts?</AlertDialogTitle>
          <AlertDialogDescription>
            Opsis will connect to the internet to show the list of free fonts, and to download the
            ones you pick. Each font is saved to your library and works offline afterwards. Nothing
            about you or your projects is sent.
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

function RemoveFontDialog({ font, onClose }: { font: FontChoice; onClose: () => void }) {
  const { family } = font.def;
  const remove = async () => {
    await attempt(() => useFontLibrary.getState().remove(font), `Couldn't remove ${family}`);
    onClose();
  };

  return (
    <AlertDialog open onOpenChange={(o) => !o && onClose()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Remove {family}?</AlertDialogTitle>
          <AlertDialogDescription>
            It's removed from your library. Projects that already use it keep their own copy.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction variant="destructive" onClick={() => void remove()}>
            Remove
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

type PanelDialog =
  | { kind: "review"; review: FontReview }
  | { kind: "installed"; fonts: InstalledFamily[] }
  | { kind: "ask-online" }
  | { kind: "online" }
  | { kind: "remove"; font: FontChoice };

export function FontsPanel() {
  const library = useFontLibrary((s) => s.library);
  const [dialog, setDialog] = useState<PanelDialog | null>(null);
  const closeIfStill = (kind: PanelDialog["kind"]) =>
    setDialog((d) => (d?.kind === kind ? null : d));

  useEffect(() => {
    void refreshLibrary();
  }, []);

  const addFiles = () =>
    attempt(async () => {
      const fonts = await commands.inspectFontFiles(null);
      if (!fonts) return;
      if (!fonts.families.length) throw new Error("No fonts found in those files.");
      setDialog({ kind: "review", review: { fonts, installed: false } });
    });

  const openInstalled = () =>
    attempt(async () => {
      setDialog({ kind: "installed", fonts: await commands.installedFontFamilies() });
    });

  return (
    <SettingsCard
      title="Your library"
      description={`${bundledFonts().length} fonts come with Opsis. Fonts you add here work in every project; a project keeps its own copy of the fonts it uses.`}
      contentClassName="flex flex-col gap-1"
      footer={
        <>
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
        </>
      }
    >
      {library.length === 0 ? (
        <p className="text-muted-foreground text-sm">No fonts added yet.</p>
      ) : (
        library.map((choice) => (
          <LibraryRow
            key={choice.def.id}
            choice={choice}
            onRemove={() => setDialog({ kind: "remove", font: choice })}
          />
        ))
      )}
      {dialog?.kind === "review" ? (
        <ReviewDialog
          review={dialog.review}
          onClose={() => closeIfStill("review")}
          onAdded={refreshLibrary}
        />
      ) : null}
      {dialog?.kind === "installed" ? (
        <InstalledDialog
          fonts={dialog.fonts}
          onClose={() => closeIfStill("installed")}
          onPicked={(fonts) => setDialog({ kind: "review", review: { fonts, installed: true } })}
        />
      ) : null}
      {dialog?.kind === "ask-online" ? (
        <OnlineConsentDialog
          onClose={() => closeIfStill("ask-online")}
          onConnect={() => {
            onlineAllowed = true;
            setDialog({ kind: "online" });
          }}
        />
      ) : null}
      {dialog?.kind === "online" ? (
        <OnlineDialog onClose={() => closeIfStill("online")} onAdded={refreshLibrary} />
      ) : null}
      {dialog?.kind === "remove" ? (
        <RemoveFontDialog font={dialog.font} onClose={() => closeIfStill("remove")} />
      ) : null}
    </SettingsCard>
  );
}
