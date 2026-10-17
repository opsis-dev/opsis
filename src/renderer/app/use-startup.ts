import { useRouter } from "@tanstack/react-router";
import { useEffect } from "react";
import { reportError } from "@/renderer/lib/toast";
import { loadedAppSettings } from "../store/app-settings";
import { useEnvironment } from "../store/environment";
import { dispatchMenuCommand } from "./menu-commands";
import { isSetupSkipped, markStarted } from "./startup-flags";

export function useStartup(): void {
  const router = useRouter();
  useEffect(() => {
    if (router.state.location.pathname !== "/") return;
    const skipped = isSetupSkipped();
    const first = markStarted();
    void Promise.all([useEnvironment.getState().refresh(), loadedAppSettings()])
      .then(async ([env, settings]) => {
        if (env && !env.engine.complete && !skipped) {
          void router.navigate({ to: "/setup" });
          return;
        }
        if (!first) return;
        if (settings.app.onStart === "last-project") {
          const [last] = await window.demoflow.listRecentProjects();
          if (last) void router.navigate({ to: "/editor", search: { dir: last.dir } });
        } else if (settings.app.onStart === "new-project") {
          dispatchMenuCommand("project:new");
        }
      })
      .catch((e: unknown) => reportError("Could not finish starting up", e));
  }, [router]);
}
