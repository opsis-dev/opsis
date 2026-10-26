use opsis_types::shell::{MenuCommand, MenuEntry, MenuSection, WindowRole};

fn command(label: &str, command: MenuCommand, accelerator: &str) -> MenuEntry {
    MenuEntry::Command {
        label: label.to_owned(),
        command,
        accelerator: Some(accelerator.to_owned()),
    }
}

fn role(label: &str, role: WindowRole, accelerator: &str) -> MenuEntry {
    MenuEntry::Role {
        label: label.to_owned(),
        role,
        accelerator: Some(accelerator.to_owned()),
    }
}

pub fn app_menu(mac: bool) -> Vec<MenuSection> {
    let section = |label: &str, items: Vec<MenuEntry>| MenuSection {
        label: label.to_owned(),
        items,
    };
    let mut file = vec![
        command("New project…", MenuCommand::ProjectNew, "CmdOrCtrl+N"),
        command("Open project…", MenuCommand::ProjectOpen, "CmdOrCtrl+O"),
        command("Save", MenuCommand::ProjectSave, "CmdOrCtrl+S"),
        MenuEntry::Separator,
        command(
            "All projects",
            MenuCommand::ViewProjects,
            "CmdOrCtrl+Shift+P",
        ),
        command("Settings", MenuCommand::ViewSettings, "CmdOrCtrl+,"),
    ];
    if !mac {
        file.extend([
            MenuEntry::Separator,
            role("Quit", WindowRole::Quit, "Ctrl+Q"),
        ]);
    }
    vec![
        section("File", file),
        section(
            "Edit",
            vec![
                command("Undo", MenuCommand::EditUndo, "CmdOrCtrl+Z"),
                command(
                    "Redo",
                    MenuCommand::EditRedo,
                    if mac { "Cmd+Shift+Z" } else { "Ctrl+Y" },
                ),
                MenuEntry::Separator,
                role("Cut", WindowRole::Cut, "CmdOrCtrl+X"),
                role("Copy", WindowRole::Copy, "CmdOrCtrl+C"),
                role("Paste", WindowRole::Paste, "CmdOrCtrl+V"),
                command("Select all", MenuCommand::EditSelectAll, "CmdOrCtrl+A"),
                MenuEntry::Separator,
                command("Add node…", MenuCommand::EditAddNode, "CmdOrCtrl+K"),
                command("Group selection", MenuCommand::EditGroup, "CmdOrCtrl+G"),
                command(
                    "Tidy up layout",
                    MenuCommand::EditLayout,
                    "CmdOrCtrl+Shift+L",
                ),
            ],
        ),
        section(
            "Run",
            vec![
                command("Run", MenuCommand::RunStart, "CmdOrCtrl+Enter"),
                command("Stop run", MenuCommand::RunCancel, "CmdOrCtrl+."),
            ],
        ),
        section(
            "View",
            vec![
                command("Node list", MenuCommand::ViewToggleLeft, "CmdOrCtrl+B"),
                command(
                    "Preview, timeline, mixer and run",
                    MenuCommand::ViewToggleBottom,
                    "CmdOrCtrl+J",
                ),
                command("Inspector", MenuCommand::ViewToggleRight, "CmdOrCtrl+Alt+B"),
                command(
                    "Minimap",
                    MenuCommand::ViewToggleMinimap,
                    "CmdOrCtrl+Shift+M",
                ),
                MenuEntry::Separator,
                command("Preview", MenuCommand::ViewFocusPreview, "CmdOrCtrl+1"),
                command("Timeline", MenuCommand::ViewFocusTimeline, "CmdOrCtrl+2"),
                command("Mixer", MenuCommand::ViewFocusMixer, "CmdOrCtrl+3"),
                command("Run", MenuCommand::ViewFocusRun, "CmdOrCtrl+4"),
                MenuEntry::Separator,
                role("Reload", WindowRole::Reload, "CmdOrCtrl+R"),
                role(
                    "Developer tools",
                    WindowRole::ToggleDevTools,
                    if mac { "Alt+Cmd+I" } else { "Ctrl+Shift+I" },
                ),
                MenuEntry::Separator,
                role("Actual size", WindowRole::ResetZoom, "CmdOrCtrl+0"),
                role("Zoom in", WindowRole::ZoomIn, "CmdOrCtrl+="),
                role("Zoom out", WindowRole::ZoomOut, "CmdOrCtrl+-"),
                MenuEntry::Separator,
                role(
                    "Full screen",
                    WindowRole::ToggleFullscreen,
                    if mac { "Ctrl+Cmd+F" } else { "F11" },
                ),
            ],
        ),
    ]
}
