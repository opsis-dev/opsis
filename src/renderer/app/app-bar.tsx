import { Copy, Menu, Minus, Square, X } from "lucide-react";
import { useEffect, useState, type ComponentType, type ReactNode } from "react";
import type { MenuCommand, WindowState } from "@/shared/ipc";
import logoUrl from "../../../brand/logo.svg";
import {
  acceleratorLabel,
  appMenu,
  matchesAccelerator,
  menuCommandEntry,
  type MenuEntry,
  type WindowRole,
} from "@/shared/menu";
import { Button } from "@/renderer/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/renderer/components/ui/dropdown-menu";
import {
  Menubar,
  MenubarContent,
  MenubarGroup,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarShortcut,
  MenubarTrigger,
} from "@/renderer/components/ui/menubar";
import { Badge } from "@/renderer/components/ui/badge";
import { Separator } from "@/renderer/components/ui/separator";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { useAppSettings } from "../store/app-settings";
import { cn } from "@/renderer/lib/utils";
import { isEditingElement } from "@/renderer/lib/dom";
import { isMac, platform } from "@/renderer/lib/platform";

export const shortcutLabel = (accelerator: string) => acceleratorLabel(accelerator, platform);
export const commandLabel = (cmd: MenuCommand): string =>
  menuCommandEntry(platform, cmd)?.label ?? cmd;

export function commandShortcut(cmd: MenuCommand): string | null {
  const accelerator = menuCommandEntry(platform, cmd)?.accelerator;
  return accelerator ? shortcutLabel(accelerator) : null;
}
const SECTIONS = appMenu(platform);

type MenuItem = Exclude<MenuEntry, { kind: "separator" }>;

function run(entry: MenuItem) {
  if (entry.kind === "command") window.demoflow.dispatchMenuCommand(entry.command);
  else void window.demoflow.windowRole(entry.role);
}

function groups(items: MenuEntry[]): MenuItem[][] {
  const out: MenuItem[][] = [];
  let current: MenuItem[] = [];
  for (const item of items) {
    if (item.kind !== "separator") current.push(item);
    else if (current.length) {
      out.push(current);
      current = [];
    }
  }
  return current.length ? [...out, current] : out;
}

type MenuParts = {
  Group: ComponentType<{ children: ReactNode }>;
  Separator: ComponentType;
  Item: ComponentType<{ onClick: () => void; children: ReactNode }>;
  Shortcut: ComponentType<{ children: ReactNode }>;
};

const MENUBAR_PARTS: MenuParts = {
  Group: MenubarGroup,
  Separator: MenubarSeparator,
  Item: MenubarItem,
  Shortcut: MenubarShortcut,
};

const DROPDOWN_PARTS: MenuParts = {
  Group: DropdownMenuGroup,
  Separator: DropdownMenuSeparator,
  Item: DropdownMenuItem,
  Shortcut: DropdownMenuShortcut,
};

function SectionItems({ items, parts }: { items: MenuEntry[]; parts: MenuParts }) {
  const { Group, Separator: GroupSeparator, Item, Shortcut } = parts;
  return groups(items).map((group, i) => (
    <Group key={i}>
      {i > 0 ? <GroupSeparator /> : null}
      {group.map((entry) => (
        <Item key={entry.label} onClick={() => run(entry)}>
          {entry.label}
          {entry.accelerator ? <Shortcut>{shortcutLabel(entry.accelerator)}</Shortcut> : null}
        </Item>
      ))}
    </Group>
  ));
}

function WideMenu() {
  return (
    <Menubar className="hidden h-8 border-0 bg-transparent p-0 xl:flex">
      {SECTIONS.map((section) => (
        <MenubarMenu key={section.label}>
          <MenubarTrigger className="px-2 font-normal">{section.label}</MenubarTrigger>
          <MenubarContent className="min-w-56">
            <SectionItems items={section.items} parts={MENUBAR_PARTS} />
          </MenubarContent>
        </MenubarMenu>
      ))}
    </Menubar>
  );
}

function CompactMenu() {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={<Button variant="ghost" size="icon-sm" aria-label="Menu" className="xl:hidden" />}
      >
        <Menu />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-48">
        <DropdownMenuGroup>
          {SECTIONS.map((section) => (
            <DropdownMenuSub key={section.label}>
              <DropdownMenuSubTrigger>{section.label}</DropdownMenuSubTrigger>
              <DropdownMenuSubContent className="min-w-56">
                <SectionItems items={section.items} parts={DROPDOWN_PARTS} />
              </DropdownMenuSubContent>
            </DropdownMenuSub>
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

const NATIVE_WHILE_EDITING = new Set([
  "cut",
  "copy",
  "paste",
  "selectAll",
  "edit:undo",
  "edit:redo",
]);

export function useMenuShortcuts(): void {
  useEffect(() => {
    if (isMac) return;
    const entries = SECTIONS.flatMap((s) => s.items).filter(
      (e): e is MenuItem & { accelerator: string } => e.kind !== "separator" && !!e.accelerator,
    );
    const onKey = (e: KeyboardEvent) => {
      if (e.defaultPrevented) return;
      const entry = entries.find((x) => matchesAccelerator(x.accelerator, e, platform));
      if (!entry) return;
      const id = entry.kind === "command" ? entry.command : entry.role;
      if (isEditingElement(document.activeElement) && NATIVE_WHILE_EDITING.has(id)) return;
      e.preventDefault();
      run(entry);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}

function useWindowState(): WindowState {
  const [state, setState] = useState<WindowState>({
    maximized: false,
    fullscreen: false,
    focused: true,
  });
  useEffect(() => {
    if (isMac) return;
    void window.demoflow.getWindowState().then(setState);
    return window.demoflow.onWindowState(setState);
  }, []);
  return state;
}

function WindowButton({
  role,
  label,
  className = "hover:bg-accent",
  children,
}: {
  role: WindowRole;
  label: string;
  className?: string;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      className={cn(
        "text-muted-foreground hover:text-foreground grid h-full w-12 place-items-center transition-colors",
        className,
      )}
      onClick={() => void window.demoflow.windowRole(role)}
    >
      {children}
    </button>
  );
}

function WindowControls() {
  const { maximized } = useWindowState();
  return (
    <div className="flex h-full shrink-0 items-stretch">
      <WindowButton role="minimize" label="Minimize">
        <Minus className="size-4" strokeWidth={1.5} />
      </WindowButton>
      <WindowButton role="toggleMaximize" label={maximized ? "Restore" : "Maximize"}>
        {maximized ? (
          <Copy className="size-3.5 -scale-x-100" strokeWidth={1.5} />
        ) : (
          <Square className="size-3.5" strokeWidth={1.5} />
        )}
      </WindowButton>
      <WindowButton role="close" label="Close" className="hover:bg-destructive hover:text-white">
        <X className="size-4" strokeWidth={1.5} />
      </WindowButton>
    </div>
  );
}

export function WindowBorder() {
  const { maximized, fullscreen, focused } = useWindowState();
  if (isMac || maximized || fullscreen) return null;
  return (
    <div
      aria-hidden
      className={cn(
        "pointer-events-none fixed inset-0 z-50 border",
        focused ? "border-white/12" : "border-white/7",
      )}
    />
  );
}

export function AppBar({
  children,
  className,
  menu = true,
}: {
  children?: ReactNode;
  className?: string;
  menu?: boolean;
}) {
  return (
    <header
      className={cn(
        "app-drag bg-sidebar text-sidebar-foreground flex h-12 shrink-0 items-center border-b",
        isMac ? "pr-3 pl-21" : "pl-2",
        className,
      )}
    >
      <div className="flex h-full min-w-0 flex-1 items-center gap-2">
        <img src={logoUrl} alt="Demoflow" className="mx-1 size-5 shrink-0" draggable={false} />
        {isMac || !menu ? null : (
          <>
            <WideMenu />
            <CompactMenu />
            {children ? (
              <Separator orientation="vertical" className="mx-1 h-5 data-vertical:self-center" />
            ) : null}
          </>
        )}
        {children}
      </div>
      <ApiBadge />
      {isMac ? null : (
        <>
          <Separator orientation="vertical" className="mx-3 h-5 data-vertical:self-center" />
          <WindowControls />
        </>
      )}
    </header>
  );
}

function ApiBadge() {
  const on = useAppSettings((s) => s.settings?.api.enabled);
  const status = useAppSettings((s) => s.api);
  if (!on || !status) return null;
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <button
            type="button"
            className="ml-2 shrink-0"
            onClick={() => void window.demoflow.openSettings("api")}
          />
        }
      >
        <Badge variant={status.listening ? "secondary" : "destructive"}>API</Badge>
      </TooltipTrigger>
      <TooltipContent>
        {status.listening
          ? "Scripts and agents on this computer can use the open project."
          : "The API is switched on but isn't running. Open its settings to see why."}
      </TooltipContent>
    </Tooltip>
  );
}

export function PageBar({ title }: { title: string }) {
  return (
    <AppBar>{title ? <span className="text-muted-foreground text-sm">{title}</span> : null}</AppBar>
  );
}
