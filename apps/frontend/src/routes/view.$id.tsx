import { createFileRoute } from "@tanstack/react-router";
import { AppBar } from "../app/app-bar";
import { StreamView } from "../app/stream-view";

function ViewRoute() {
  const { id } = Route.useParams();
  return (
    <div className="bg-background flex h-screen flex-col">
      <AppBar menu={false} />
      <StreamView id={id} />
    </div>
  );
}

export const Route = createFileRoute("/view/$id")({ component: ViewRoute });
