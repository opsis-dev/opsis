import { FileArchive, Globe, HardDrive, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { BUNDLED_FONTS } from "@/shared/fonts/bundled.generated";
import type { FontFamilyDef } from "@/shared/fonts/types";
import { errorMessage } from "@/shared/errors";
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
import { describeFaces } from "./font-labels";
import { InstalledDialog } from "./installed-dialog";
import { useFontLibrary, usePreviewFamily } from "./library";
import { installedFonts, type LocalFontData } from "./local-fonts";
import { OnlineDialog } from "./online-dialog";
import { ReviewDialog, type FontReview } from "./review-dialog";

const SOURCE_LABEL: Record<FontFamilyDef["source"], string> = {
  bundled: "Built in",
  online: "Downloaded",
  file: "From files",
  installed: "From this computer",
};

const refreshLibrary = () =>
  useFontLibrary
    .getState()
    .refresh()
    .catch((e: unknown) =>
      toast.error("Could not read your font library", { description: errorMessage(e) }),
    );

function Sample({ def }: { def: FontFamilyDef }) {
  const family = usePreviewFamily(def);
  return (
    <span className="truncate text-base" style={{ fontFamily: `"${family}", sans-serif` }}>
      {def.family}
    </span>
  );
}

function LibraryRow({ def, onRemove }: { def: FontFamilyDef; onRemove: () => void }) {
  return (
    <div className="flex items-center gap-3 rounded-md px-2 py-1.5">
      <div className="flex min-w-0 flex-1 flex-col">
        <Sample def={def} />
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
    try {
      await window.demoflow.removeFont(def.id);
      await refreshLibrary();
    } catch (e) {
      toast.error(`Could not remove ${def.family}`, { description: errorMessage(e) });
    } finally {
      onClose();
    }
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

export function FontsPanel() {
  const library = useFontLibrary((s) => s.library);
  const onlineAllowed = useFontLibrary((s) => s.onlineAllowed);
  const [review, setReview] = useState<FontReview | null>(null);
  const [installed, setInstalled] = useState<LocalFontData[] | null>(null);
  const [askOnline, setAskOnline] = useState(false);
  const [online, setOnline] = useState(false);
  const [removing, setRemoving] = useState<FontFamilyDef | null>(null);

  useEffect(() => {
    void refreshLibrary();
  }, []);

  const addFiles = async () => {
    try {
      const fonts = await window.demoflow.inspectFontFiles();
      if (!fonts) return;
      if (!fonts.families.length) return void toast.error("No fonts found in those files.");
      setReview({ fonts, installed: false });
    } catch (error) {
      toast.error(errorMessage(error));
    }
  };

  const openInstalled = async () => {
    try {
      setInstalled(await installedFonts());
    } catch (error) {
      toast.error(errorMessage(error));
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
            <LibraryRow key={def.id} def={def} onRemove={() => setRemoving(def)} />
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

      {review ? (
        <ReviewDialog
          review={review}
          onClose={() => setReview(null)}
          onAdded={() => void refreshLibrary()}
        />
      ) : null}
      {installed ? (
        <InstalledDialog
          fonts={installed}
          onClose={() => setInstalled(null)}
          onPicked={(fonts) => {
            setInstalled(null);
            setReview({ fonts, installed: true });
          }}
        />
      ) : null}
      <OnlineConsentDialog
        open={askOnline}
        onOpenChange={setAskOnline}
        onConnect={() => {
          useFontLibrary.getState().allowOnline();
          setAskOnline(false);
          setOnline(true);
        }}
      />
      <OnlineDialog
        open={online}
        onClose={() => setOnline(false)}
        onAdded={() => void refreshLibrary()}
      />
      <RemoveFontDialog font={removing} onClose={() => setRemoving(null)} />
    </Card>
  );
}
