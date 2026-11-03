import { createFileRoute } from "@tanstack/react-router";
import { AppBar } from "../app/app-bar";
import { TerminalWindow } from "../app/terminal-window";

function TerminalRoute() {
  const { id } = Route.useParams();
  return (
    <div className="bg-background flex h-screen flex-col">
      <AppBar menu={false} />
      <TerminalWindow id={id} />
    </div>
  );
}

export const Route = createFileRoute("/terminal/$id")({ component: TerminalRoute });
