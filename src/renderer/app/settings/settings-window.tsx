import { useNavigate } from "@tanstack/react-router";
import {
  Clapperboard,
  Cpu,
  FolderKanban,
  Info,
  Plug,
  Scale,
  SlidersHorizontal,
  Type,
  Video,
  type LucideIcon,
} from "lucide-react";
import { useEffect, type ReactNode } from "react";
import type { SettingsPage } from "@/shared/ipc";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
} from "@/renderer/components/ui/sidebar";
import { AppBar } from "../app-bar";
import { FontsPanel } from "../../fonts/fonts-panel";
import { EnginePanel } from "../setup";
import { ApiPage } from "./api-page";
import { LicencesPage } from "./licences-page";
import { AboutPage, ExportPage, GeneralPage, ProjectsPage, RecordingPage } from "./pages";

const PAGES: Array<{ id: SettingsPage; label: string; icon: LucideIcon; description: string }> = [
  {
    id: "general",
    label: "General",
    icon: SlidersHorizontal,
    description: "What Demoflow does when it opens and before it closes.",
  },
  {
    id: "projects",
    label: "Projects",
    icon: FolderKanban,
    description: "Where new projects go.",
  },
  {
    id: "engine",
    label: "Engine",
    icon: Cpu,
    description: "The recording browser and video tools.",
  },
  {
    id: "recording",
    label: "Recording",
    icon: Video,
    description: "Traces of each take and the live view while a run is going.",
  },
  {
    id: "fonts",
    label: "Fonts",
    icon: Type,
    description: "Your font library. Projects copy the fonts they use.",
  },
  {
    id: "export",
    label: "Export licence",
    icon: Clapperboard,
    description: "The licence for exporting videos.",
  },
  {
    id: "api",
    label: "API",
    icon: Plug,
    description: "Let scripts and AI agents on this computer read, edit and run the open project.",
  },
  {
    id: "licences",
    label: "Licences",
    icon: Scale,
    description: "Licences of Demoflow and everything it includes.",
  },
  { id: "about", label: "About", icon: Info, description: "Versions and folders." },
];

const isPage = (v: string): v is SettingsPage => PAGES.some((p) => p.id === v);

function Page({ page }: { page: SettingsPage }): ReactNode {
  switch (page) {
    case "general":
      return <GeneralPage />;
    case "projects":
      return <ProjectsPage />;
    case "engine":
      return <EnginePanel title="Status" />;
    case "recording":
      return <RecordingPage />;
    case "fonts":
      return <FontsPanel />;
    case "export":
      return <ExportPage />;
    case "api":
      return <ApiPage />;
    case "licences":
      return <LicencesPage />;
    case "about":
      return <AboutPage />;
  }
}

export function SettingsWindow({ page: requested }: { page: string }) {
  const navigate = useNavigate();
  const page = isPage(requested) ? requested : "general";
  const meta = PAGES.find((p) => p.id === page)!;

  useEffect(() => window.demoflow.setTitle("Settings — Demoflow"), []);

  useEffect(
    () =>
      window.demoflow.onMenuCommand((command) => {
        if (command !== "view:settings") window.demoflow.sendMenuToEditor(command);
      }),
    [],
  );

  return (
    <div className="flex h-screen flex-col">
      <AppBar menu={false}>
        <span className="truncate font-semibold">Settings</span>
      </AppBar>
      <SidebarProvider className="min-h-0 flex-1">
        <Sidebar collapsible="none" className="w-56 border-r">
          <SidebarContent>
            <SidebarGroup>
              <SidebarGroupContent>
                <SidebarMenu>
                  {PAGES.map((p) => (
                    <SidebarMenuItem key={p.id}>
                      <SidebarMenuButton
                        isActive={p.id === page}
                        onClick={() =>
                          void navigate({ to: "/settings", search: { page: p.id }, replace: true })
                        }
                      >
                        <p.icon />
                        <span>{p.label}</span>
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                  ))}
                </SidebarMenu>
              </SidebarGroupContent>
            </SidebarGroup>
          </SidebarContent>
        </Sidebar>
        <ScrollArea className="bg-background min-w-0 flex-1">
          <div className="mx-auto flex max-w-3xl flex-col gap-6 px-8 py-8">
            <header className="flex flex-col gap-1">
              <h1 className="text-xl font-semibold tracking-tight">{meta.label}</h1>
              <p className="text-muted-foreground text-sm">{meta.description}</p>
            </header>
            <Page page={page} />
          </div>
        </ScrollArea>
      </SidebarProvider>
    </div>
  );
}
