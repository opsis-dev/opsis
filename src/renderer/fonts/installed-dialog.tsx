import { useMemo, useState } from "react";
import { toast } from "sonner";
import type { FontInstalledFace, InspectedFonts } from "@/shared/ipc";
import { errorMessage } from "@/shared/errors";
import { plural } from "@/shared/format";
import type { LocalFontData } from "./local-fonts";
import { BusyCount, MAX_LISTED, SearchListDialog } from "./search-list-dialog";

async function readFaces(faces: LocalFontData[]): Promise<FontInstalledFace[]> {
  const data: FontInstalledFace[] = [];
  for (const face of faces)
    data.push({
      postscriptName: face.postscriptName,
      data: await (await face.blob()).arrayBuffer(),
    });
  return data;
}

function groupByFamily(fonts: LocalFontData[]): Array<[string, LocalFontData[]]> {
  const map = new Map<string, LocalFontData[]>();
  for (const f of fonts) map.set(f.family, [...(map.get(f.family) ?? []), f]);
  return [...map.entries()].toSorted(([a], [b]) => a.localeCompare(b));
}

export function InstalledDialog({
  fonts,
  onClose,
  onPicked,
}: {
  fonts: LocalFontData[];
  onClose: () => void;
  onPicked: (fonts: InspectedFonts) => void;
}) {
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const families = useMemo(() => groupByFamily(fonts), [fonts]);
  const q = query.toLowerCase();
  const shown = families.filter(([name]) => name.toLowerCase().includes(q));

  const pick = async (family: string, faces: LocalFontData[]) => {
    setBusy(family);
    try {
      onPicked(await window.demoflow.inspectInstalledFonts(await readFaces(faces)));
    } catch (error) {
      toast.error(errorMessage(error));
    } finally {
      setBusy(null);
    }
  };

  return (
    <SearchListDialog
      open
      onClose={onClose}
      title="Add an installed font"
      description={`${families.length} font families are installed on this computer. Pick one to copy into your library.`}
      placeholder="Search installed fonts…"
      query={query}
      onQuery={setQuery}
      matches={shown.length}
    >
      {shown.slice(0, MAX_LISTED).map(([name, faces]) => (
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
          <BusyCount busy={busy === name}>{plural(faces.length, "style")}</BusyCount>
        </button>
      ))}
    </SearchListDialog>
  );
}
