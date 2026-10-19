import { Spinner } from "@/renderer/components/ui/spinner";
import { useFreshEnvironment } from "../../store/environment";
import { SettingsCard } from "./settings-card";

export function AboutPage() {
  const env = useFreshEnvironment();
  if (!env) return <Spinner />;
  const rows: Array<[string, string]> = [
    ["App", `${env.versions.app} (${env.edition})`],
    ["Electron", env.versions.electron],
    ["Chromium", env.versions.chrome],
    ["Node", env.versions.node],
    ["Platform", `${env.platform} / ${env.arch}`],
    ["Engine folder", env.engine.dir],
    ["Recording browser", env.engine.headlessShell ?? "Not set up"],
    ["Video tools", env.engine.ffmpeg ?? "Not set up"],
    ["Video inspector", env.engine.ffprobe ?? "Not set up"],
    ["App data", env.userData],
  ];
  return (
    <SettingsCard
      title={`Opsis ${env.versions.app}`}
      description="Everything runs on this computer."
    >
      <dl className="grid grid-cols-[10rem_1fr] gap-x-4 gap-y-2 text-sm">
        {rows.map(([k, v]) => (
          <div key={k} className="contents">
            <dt className="text-muted-foreground">{k}</dt>
            <dd className="font-mono text-xs leading-5 break-all">{v}</dd>
          </div>
        ))}
      </dl>
    </SettingsCard>
  );
}
