use std::env;
use std::path::{Path, PathBuf};

use qt_build_utils::{MocArguments, QtBuild, QtInstallationQMake};

const QT_MODULES: [&str; 5] = ["Core", "Gui", "Widgets", "Svg", "Test"];
const QT_MIN_MINOR: u64 = 12;
const KDDW_VERSION: &str = "2.5.1";
const QT_VERSION: &str = "6.12.0";

const DEFINES: [&str; 6] = [
    "KDDW_FRONTEND_QT",
    "KDDW_FRONTEND_QTWIDGETS",
    "QT_WIDGETS_LIB",
    "QT_GUI_LIB",
    "QT_CORE_LIB",
    "QT_TESTLIB_LIB",
];

const MOC_HEADERS: [&str; 25] = [
    "event_hub.h",
    "widgets.h",
    "window_chrome.h",
    "dock_style.h",
    "graph_canvas.h",
    "node_list.h",
    "inspector_panel.h",
    "start_screen.h",
    "settings_window.h",
    "main_window.h",
    "scenario.h",
    "new_project_dialog.h",
    "var_text.h",
    "project_panel.h",
    "top_bar.h",
    "color_picker.h",
    "form_hold.h",
    "playback_clock.h",
    "timeline_canvas.h",
    "timeline_panel.h",
    "mixer_controls.h",
    "mixer_panel.h",
    "gradient_field.h",
    "visual_fields.h",
    "marks_field.h",
];

const SOURCES: [&str; 32] = [
    "app.cc",
    "event_hub.cc",
    "widgets.cc",
    "window_chrome.cc",
    "dock_style.cc",
    "graph_canvas.cc",
    "node_list.cc",
    "inspector_panel.cc",
    "start_screen.cc",
    "settings_window.cc",
    "main_window.cc",
    "scenario.cc",
    "platform.cc",
    "new_project_dialog.cc",
    "motion.cc",
    "look_style.cc",
    "var_text.cc",
    "project_panel.cc",
    "top_bar.cc",
    "e2e_steps.cc",
    "list_fields.cc",
    "color_picker.cc",
    "form_hold.cc",
    "gradient_field.cc",
    "visual_fields.cc",
    "marks_field.cc",
    "keyframe_fields.cc",
    "playback_clock.cc",
    "timeline_canvas.cc",
    "timeline_panel.cc",
    "mixer_controls.cc",
    "mixer_panel.cc",
];

fn main() {
    let manifest =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR"));
    let home = PathBuf::from(env::var("HOME").expect("HOME is set"));
    let qt_prefix = env::var("OPSIS_QT").map_or_else(
        |_| {
            home.join(".cache/opsis-qt")
                .join(QT_VERSION)
                .join(qt_folder())
        },
        PathBuf::from,
    );
    let installation = QtInstallationQMake::try_from(qt_prefix.join("bin").join("qmake6"))
        .unwrap_or_else(|error| {
            panic!(
                "Qt 6.12 is needed in {} (bun run qt:install, or set OPSIS_QT): {error}",
                qt_prefix.display()
            )
        });
    let mut qt = QtBuild::with_installation(
        Box::new(installation),
        QT_MODULES.iter().map(|m| (*m).to_owned()).collect(),
    );
    let version = qt.version();
    assert!(
        version.major == 6 && version.minor >= QT_MIN_MINOR,
        "Qt 6.12 or newer is needed, found {version}"
    );

    let kddw = build_kddw(&qt_prefix, &home);
    let cpp = manifest.join("cpp");
    let out = PathBuf::from(env::var("OUT_DIR").expect("Cargo sets OUT_DIR"));
    let bridge_include = out.join("cxxbridge").join("include");
    let crate_include = out.join("cxxbridge").join("crate");
    let mut includes = qt.include_paths();
    includes.push(kddw.join("include").join("kddockwidgets-qt6"));
    includes.push(cpp.clone());
    includes.push(bridge_include);
    includes.push(crate_include);

    let mut build = cxx_build::bridge("src/bridge.rs");
    let moc = qt.moc();
    for header in MOC_HEADERS {
        let path = cpp.join(header);
        let products = moc.compile(
            &path,
            MocArguments::default().include_paths(includes.clone()),
        );
        build.file(products.cpp);
    }
    for source in SOURCES {
        build.file(cpp.join(source));
    }
    let resources = qt
        .rcc()
        .compile(manifest.join("resources").join("resources.qrc"));
    if let Some(file) = resources.file {
        build.file(file);
    }
    build
        .std("c++17")
        .includes(&includes)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-deprecated-declarations");
    for define in DEFINES {
        build.define(define, None);
    }
    write_compile_commands(&out, &cpp, &includes);
    let kddw_lib = kddw.join("lib");
    let link_args = if static_qt(&qt_prefix) {
        build.define("KDDOCKWIDGETS_STATICLIB", None);
        build.compile("opsis_qt");
        let imports = cc::Build::new()
            .cpp(true)
            .std("c++17")
            .includes(&includes)
            .file(static_plugin_imports(&out))
            .compile_intermediates();
        static_link_args(&qt_prefix, &kddw_lib, &imports)
    } else {
        qt.cargo_link_libraries(&mut build);
        build.compile("opsis_qt");
        println!("cargo:rustc-link-search=native={}", kddw_lib.display());
        println!("cargo:rustc-link-lib=dylib=kddockwidgets-qt6");
        vec![
            format!("-Wl,-rpath,{}", kddw_lib.display()),
            format!("-Wl,-rpath,{}", qt_prefix.join("lib").display()),
        ]
    };
    let args_file = out.join("link-args.txt");
    std::fs::write(&args_file, link_args.join("\n")).expect("writing the link arguments");
    println!("cargo::metadata=link_args={}", args_file.display());
    for path in ["cpp", "resources", "src/bridge.rs"] {
        println!("cargo:rerun-if-changed={}", manifest.join(path).display());
    }
    println!("cargo:rerun-if-env-changed=OPSIS_QT");
    println!("cargo:rerun-if-env-changed=OPSIS_KDDW_SOURCE");
}

fn build_kddw(qt_prefix: &Path, home: &Path) -> PathBuf {
    let source = env::var("OPSIS_KDDW_SOURCE").map_or_else(
        |_| home.join(format!(".cache/opsis-qt/kddockwidgets-{KDDW_VERSION}")),
        PathBuf::from,
    );
    assert!(
        source.join("CMakeLists.txt").exists(),
        "KDDockWidgets {KDDW_VERSION} is needed in {} (bun run qt:install, or set OPSIS_KDDW_SOURCE)",
        source.display()
    );
    cmake::Config::new(&source)
        .define("CMAKE_PREFIX_PATH", qt_prefix)
        .define("KDDockWidgets_FRONTENDS", "qtwidgets")
        .define("KDDockWidgets_EXAMPLES", "OFF")
        .define("KDDockWidgets_NO_SPDLOG", "ON")
        .define("KDDockWidgets_TESTS", "OFF")
        .define(
            "KDDockWidgets_STATIC",
            if static_qt(qt_prefix) { "ON" } else { "OFF" },
        )
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        .profile("Release")
        .build()
}

fn static_qt(qt_prefix: &Path) -> bool {
    qt_prefix.join("lib").join("libQt6Core.a").exists()
}

fn static_plugin_imports(out: &Path) -> PathBuf {
    let text: String = std::iter::once("#include <QtPlugin>\n".to_owned())
        .chain(
            STATIC_PLUGINS
                .iter()
                .map(|(_, _, class)| format!("Q_IMPORT_PLUGIN({class})\n")),
        )
        .collect();
    let path = out.join("static_plugins.cc");
    std::fs::write(&path, text).expect("writing the plugin imports");
    path
}

#[derive(Debug, Default)]
struct StaticLinks {
    group: Vec<String>,
    system: Vec<String>,
}

impl StaticLinks {
    fn push(list: &mut Vec<String>, item: String) {
        if !list.contains(&item) {
            list.push(item);
        }
    }

    fn archive(&mut self, archive: &Path, qt_prefix: &Path) {
        Self::push(&mut self.group, archive.to_string_lossy().into_owned());
        let prl = archive.with_extension("prl");
        let Ok(text) = std::fs::read_to_string(&prl) else {
            return;
        };
        let lib = qt_prefix.join("lib").to_string_lossy().into_owned();
        let plugins = qt_prefix.join("plugins").to_string_lossy().into_owned();
        let prefix = qt_prefix.to_string_lossy().into_owned();
        for line in text.lines() {
            let Some(libs) = line.strip_prefix("QMAKE_PRL_LIBS = ") else {
                continue;
            };
            let libs = libs
                .replace("$$[QT_INSTALL_LIBS]", &lib)
                .replace("$$[QT_INSTALL_PLUGINS]", &plugins)
                .replace("$$[QT_INSTALL_PREFIX]", &prefix);
            for token in libs.split_whitespace() {
                let extension = Path::new(token).extension().and_then(|e| e.to_str());
                if matches!(extension, Some("a" | "o")) {
                    Self::push(&mut self.group, token.to_owned());
                } else {
                    Self::push(&mut self.system, token.to_owned());
                }
            }
        }
    }
}

const STATIC_PLUGINS: [(&str, &str, &str); 9] = [
    ("platforms", "qxcb", "QXcbIntegrationPlugin"),
    ("platforms", "qwayland", "QWaylandIntegrationPlugin"),
    ("platforms", "qoffscreen", "QOffscreenIntegrationPlugin"),
    (
        "wayland-shell-integration",
        "xdg-shell",
        "QWaylandXdgShellIntegrationPlugin",
    ),
    (
        "wayland-decoration-client",
        "bradient",
        "QWaylandBradientDecorationPlugin",
    ),
    (
        "platforminputcontexts",
        "composeplatforminputcontextplugin",
        "QComposePlatformInputContextPlugin",
    ),
    (
        "platforminputcontexts",
        "ibusplatforminputcontextplugin",
        "QIbusPlatformInputContextPlugin",
    ),
    (
        "platformthemes",
        "qxdgdesktopportal",
        "QXdgDesktopPortalThemePlugin",
    ),
    ("platforms", "qminimal", "QMinimalIntegrationPlugin"),
];

fn static_link_args(qt_prefix: &Path, kddw_lib: &Path, imports: &[PathBuf]) -> Vec<String> {
    let mut links = StaticLinks::default();
    for object in imports {
        StaticLinks::push(&mut links.group, object.to_string_lossy().into_owned());
    }
    links.archive(&kddw_lib.join("libkddockwidgets-qt6.a"), qt_prefix);
    for (folder, name, _) in STATIC_PLUGINS {
        let archive = qt_prefix
            .join("plugins")
            .join(folder)
            .join(format!("lib{name}.a"));
        assert!(
            archive.exists(),
            "the static Qt has no {}",
            archive.display()
        );
        links.archive(&archive, qt_prefix);
    }
    for module in QT_MODULES {
        links.archive(
            &qt_prefix.join("lib").join(format!("libQt6{module}.a")),
            qt_prefix,
        );
    }
    std::iter::once("-Wl,--start-group".to_owned())
        .chain(links.group)
        .chain(std::iter::once("-Wl,--end-group".to_owned()))
        .chain(links.system)
        .collect()
}

fn qt_folder() -> &'static str {
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    match (os.as_str(), arch.as_str()) {
        ("macos", _) => "macos",
        ("windows", _) => "msvc2022_64",
        (_, "aarch64") => "gcc_arm64",
        _ => "gcc_64",
    }
}

fn json_string(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    for ch in text.chars() {
        match ch {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            _ => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}

fn write_compile_commands(out: &Path, cpp: &Path, includes: &[PathBuf]) {
    let mut arguments = vec!["c++".to_owned(), "-std=c++17".to_owned()];
    arguments.extend(includes.iter().map(|dir| format!("-I{}", dir.display())));
    arguments.extend(DEFINES.iter().map(|define| format!("-D{define}")));
    let entries: Vec<String> = SOURCES
        .iter()
        .map(|source| {
            let file = cpp.join(source).to_string_lossy().into_owned();
            let args: Vec<String> = arguments
                .iter()
                .map(String::as_str)
                .chain(["-c", file.as_str()])
                .map(json_string)
                .collect();
            format!(
                "{{\"directory\": {}, \"file\": {}, \"arguments\": [{}]}}",
                json_string(&cpp.to_string_lossy()),
                json_string(&file),
                args.join(", ")
            )
        })
        .collect();
    let text = format!("[\n{}\n]\n", entries.join(",\n"));
    std::fs::write(out.join("compile_commands.json"), text).expect("writing compile_commands.json");
}
