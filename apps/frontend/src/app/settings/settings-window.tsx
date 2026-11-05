import { useNavigate, useSearch } from "@tanstack/react-router";
import {
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
import { useCallback, useEffect, type ReactNode } from "react";
import { SETTINGS_PAGES } from "@/app/shell";
import { on } from "@/app/tauri";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
} from "@/components/ui/sidebar";
import { Spinner } from "@/components/ui/spinner";
import { FontsPanel } from "@/fonts/fonts-panel";
import { commands, events, type SettingsPage, type AppSettings } from "@/generated/bindings";
import { useAppSettings } from "@/store/app-settings";
import { AppBar } from "../app-bar";
import { onMenuCommand } from "../menu-commands";
import { EnginePanel } from "../setup";
import { AboutPage } from "./about-page";
import { ApiPage } from "./api-page";
import { GeneralPage } from "./general-page";
import { LicencesPage } from "./licences-page";
import { ProjectsPage } from "./projects-page";
import { RecordingPage } from "./recording-page";

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
    description: "What Opsis does when it opens and before it closes.",
    render: (settings) => <GeneralPage app={settings.app} media={settings.media} />,
  },
  projects: {
    label: "Projects",
    icon: FolderKanban,
    description: "Where new projects go.",
    render: (settings) => <ProjectsPage projects={settings.projects} />,
  },
  engine: {
    label: "Engine",
    icon: Cpu,
    description: "The recording browser.",
    render: () => <EnginePanel title="Status" />,
  },
  recording: {
    label: "Recording",
    icon: Video,
    description: "Traces of each take and the live view while a run is going.",
    render: (settings) => <RecordingPage traces={settings.traces} live={settings.live} />,
  },
  fonts: {
    label: "Fonts",
    icon: Type,
    description: "Your font library. Projects copy the fonts they use.",
    render: () => <FontsPanel />,
  },
  api: {
    label: "API",
    icon: Plug,
    description: "Let scripts and AI agents on this computer read, edit and run the open project.",
    render: (settings) => <ApiPage api={settings.api} />,
  },
  licences: {
    label: "Licences",
    icon: Scale,
    description: "Licences of Opsis and everything it includes.",
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
  const showPage = useCallback(
    (next: SettingsPage) =>
      void navigate({ to: "/settings", search: { page: next }, replace: true }),
    [navigate],
  );

  useEffect(() => void commands.setTitle("Settings — Opsis"), []);

  useEffect(() => on(events.settingsPageShown, showPage), [showPage]);

  useEffect(
    () =>
      onMenuCommand((command) => {
        if (command !== "view:settings") void commands.sendMenuToEditor(command);
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
                        <SidebarMenuButton isActive={id === page} onClick={() => showPage(id)}>
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
