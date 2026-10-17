import { useNavigate, useSearch } from "@tanstack/react-router";
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
import { SETTINGS_PAGES, type SettingsPage } from "@/shared/ipc";
import type { AppSettings } from "@/shared/settings";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { Spinner } from "@/renderer/components/ui/spinner";
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
import { useAppSettings } from "../../store/app-settings";
import { FontsPanel } from "../../fonts/fonts-panel";
import { EnginePanel } from "../setup";
import { ApiPage } from "./api-page";
import { LicencesPage } from "./licences-page";
import { AboutPage, ExportPage, GeneralPage, ProjectsPage, RecordingPage } from "./pages";

type PageInfo = {
  label: string;
  icon: LucideIcon;
  description: string;
  render: (settings: AppSettings) => ReactNode;
};

const PAGES: Record<SettingsPage, PageInfo> = {
  general: {
    label: "General",
    icon: SlidersHorizontal,
    description: "What Demoflow does when it opens and before it closes.",
    render: (settings) => <GeneralPage settings={settings} />,
  },
  projects: {
    label: "Projects",
    icon: FolderKanban,
    description: "Where new projects go.",
    render: (settings) => <ProjectsPage settings={settings} />,
  },
  engine: {
    label: "Engine",
    icon: Cpu,
    description: "The recording browser and video tools.",
    render: () => <EnginePanel title="Status" />,
  },
  recording: {
    label: "Recording",
    icon: Video,
    description: "Traces of each take and the live view while a run is going.",
    render: (settings) => <RecordingPage settings={settings} />,
  },
  fonts: {
    label: "Fonts",
    icon: Type,
    description: "Your font library. Projects copy the fonts they use.",
    render: () => <FontsPanel />,
  },
  export: {
    label: "Export licence",
    icon: Clapperboard,
    description: "The licence for exporting videos.",
    render: (settings) => <ExportPage settings={settings} />,
  },
  api: {
    label: "API",
    icon: Plug,
    description: "Let scripts and AI agents on this computer read, edit and run the open project.",
    render: (settings) => <ApiPage settings={settings} />,
  },
  licences: {
    label: "Licences",
    icon: Scale,
    description: "Licences of Demoflow and everything it includes.",
    render: () => <LicencesPage />,
  },
  about: {
    label: "About",
    icon: Info,
    description: "Versions and folders.",
    render: () => <AboutPage />,
  },
};

export function SettingsWindow() {
  const navigate = useNavigate();
  const { page } = useSearch({ from: "/settings" });
  const settings = useAppSettings((s) => s.settings);
  const meta = PAGES[page];

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
                  {SETTINGS_PAGES.map((id) => {
                    const { icon: Icon, label } = PAGES[id];
                    return (
                      <SidebarMenuItem key={id}>
                        <SidebarMenuButton
                          isActive={id === page}
                          onClick={() =>
                            void navigate({ to: "/settings", search: { page: id }, replace: true })
                          }
                        >
                          <Icon />
                          <span>{label}</span>
                        </SidebarMenuButton>
                      </SidebarMenuItem>
                    );
                  })}
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
            {settings ? meta.render(settings) : <Spinner />}
          </div>
        </ScrollArea>
      </SidebarProvider>
    </div>
  );
}
