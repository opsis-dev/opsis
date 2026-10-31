import { Spinner } from "@/components/ui/spinner";
import { useFreshEnvironment } from "@/store/environment";
import { SettingsCard } from "./settings-card";

export function AboutPage() {
  const env = useFreshEnvironment();
  if (!env) return <Spinner />;
  const rows: Array<[string, string]> = [
    ["App", env.isPackaged ? env.versions.app : `${env.versions.app} (development)`],
    ["Tauri", env.versions.tauri],
    ["Web view", env.versions.webview],
    ["Platform", `${env.platform} / ${env.arch}`],
    ["Recording browser", env.engine.browser.path ?? "Not set up"],
    ["Engine downloads", env.engine.downloadDir],
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
