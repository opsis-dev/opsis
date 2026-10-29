import { createFileRoute } from "@tanstack/react-router";
import { useEffect } from "react";
import { AppBar } from "@/app/app-bar";
import { watchLive } from "@/app/tauri";
import { LiveBadge } from "@/editor/live-view";
import { useLiveStore } from "@/store/live";

function LiveWindow() {
  useEffect(() => watchLive(useLiveStore.getState().apply), []);
  const capture = useLiveStore((s) => (s.focusId ? s.captures[s.focusId] : undefined));
  return (
    <div className="flex h-screen flex-col bg-black">
      <AppBar menu={false} />
      <div className="relative min-h-0 flex-1">
        {capture?.frameUrl ? (
          <img
            src={capture.frameUrl}
            alt="Live recording"
            className="absolute inset-0 size-full object-contain"
          />
        ) : null}
        {capture?.frameUrl && capture.status === "live" ? (
          <LiveBadge className="absolute top-3 left-3" />
        ) : null}
      </div>
    </div>
  );
}

export const Route = createFileRoute("/live")({ component: LiveWindow });
