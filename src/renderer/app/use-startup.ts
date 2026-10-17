import { useRouter } from "@tanstack/react-router";
import { useEffect } from "react";
import { useEnvironment } from "../store/environment";
import { readStored } from "../store/view-prefs";
import { SETUP_SKIPPED_KEY } from "./setup";

const STARTED_KEY = "demoflow.started";

function firstStart(): boolean {
  const first = sessionStorage.getItem(STARTED_KEY) !== "1";
  sessionStorage.setItem(STARTED_KEY, "1");
  return first;
}

export function useStartup(): void {
  const router = useRouter();
  useEffect(() => {
    if (router.state.location.pathname !== "/") return;
    const skipped = readStored(SETUP_SKIPPED_KEY) === "1";
    const first = firstStart();
    void Promise.all([useEnvironment.getState().refresh(), window.demoflow.getSettings()]).then(
      async ([env, settings]) => {
        if (env && !env.engine.complete && !skipped) {
          void router.navigate({ to: "/setup" });
          return;
        }
        if (!first) return;
        if (settings.app.onStart === "last-project") {
          const [last] = await window.demoflow.listRecentProjects();
          if (last) void router.navigate({ to: "/editor", search: { dir: last.dir } });
        } else if (settings.app.onStart === "new-project") {
          window.demoflow.dispatchMenuCommand("project:new");
        }
      },
    );
  }, [router]);
}
