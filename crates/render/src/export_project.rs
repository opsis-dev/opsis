use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use indexmap::{IndexMap, IndexSet};
use opsis_compile::asset_ref::{STATIC_PROJECT_BASE, is_project_ref, project_rel_path};
use opsis_compile::clip::walk_items;
use opsis_compile::nodes::output::{EXPORT_FOLDER_FALLBACK, ExportCode};
use opsis_compile::text::slug_name;
use opsis_compile::vars::strings_in;
use opsis_engine::files::{read_json, write_json};
use opsis_project::paths::{to_posix_rel, to_project_ref};
use opsis_runner::context::{ExecError, ExecutorContext, ExecutorOutput};
use opsis_types::timeline::{SpecAssets, TimelineItem, TimelineSpec};
use opsis_types::values::{Artifact, ArtifactKind, CompileValue, PortValues, SrcValue};
use regex::Regex;
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::executors::RenderEngine;
use crate::film::{Stage, mixed, spec_for};

const EXPORT_MIX_STAGE: Stage = (0.0, 0.3);
const EXPORT_PEERS: [&str; 2] = ["react", "react-dom"];
const EXPORT_DEV_DEPENDENCIES: [&str; 6] = [
    "@remotion/cli",
    "@remotion/tailwind-v4",
    "tailwindcss",
    "typescript",
    "@types/react",
    "@types/node",
];
const PACKAGE_SOURCE_PREFIX: &str = "./src/";
const CANT_EXPORT_CODE: &str = "This copy of Opsis can't export code.";

const INDEX_TS: &str = r#"import { registerRoot } from "remotion";
import "./video/video.css";
import { ExportRoot } from "./export-root";

registerRoot(ExportRoot);
"#;

const EXPORT_ROOT_TSX: &str = r#"import { Composition } from "remotion";
import { DemoTimeline } from "./video/demo-timeline";
import type { TimelineSpec } from "./video/generated/spec";
import timeline from "./timeline.json";

const spec = timeline as TimelineSpec;

export const ExportRoot = () => (
  <Composition
    id="DemoTimeline"
    component={DemoTimeline}
    defaultProps={spec}
    durationInFrames={spec.durationInFrames}
    fps={spec.fps}
    width={spec.width}
    height={spec.height}
  />
);
"#;

static SOURCE_FILE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\.(ts|tsx|mts|css)$").expect("pattern"));
static IMPORT_SPECIFIER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:\bfrom\s*|\bimport\s*\(?\s*|@import\s+)["']([^"'\n]+)["']"#).expect("pattern")
});
static LOCAL_SPECIFIER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\.|/|#|@/|@opsis/|node:)").expect("pattern"));
static LAYOUT: LazyLock<SourceLayout> = LazyLock::new(|| {
    serde_json::from_str(include_str!("source-layout.generated.json"))
        .expect("source layout parses")
});

#[derive(Debug, Deserialize)]
pub struct ExportedSource {
    pub from: Vec<String>,
    pub to: String,
}

#[derive(Debug, Deserialize)]
struct ExportedPackage {
    name: String,
    folder: String,
    manifest: ExportedSource,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceLayout {
    video_source: Vec<String>,
    exported_sources: Vec<ExportedSource>,
    exported_packages: Vec<ExportedPackage>,
    dependency_manifests: Vec<ExportedSource>,
}

#[derive(Debug, Clone)]
pub struct ExportSources {
    pub root: PathBuf,
    pub packaged: bool,
}

impl ExportSources {
    fn path(&self, source: &ExportedSource) -> PathBuf {
        if self.packaged {
            self.root.join(&source.to)
        } else {
            source
                .from
                .iter()
                .fold(self.root.clone(), |path, part| path.join(part))
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct Imports {
    imports: Option<IndexMap<String, String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    dependencies: Option<IndexMap<String, String>>,
    dev_dependencies: Option<IndexMap<String, String>>,
}

fn remotion_config_ts() -> String {
    let packages: String = LAYOUT
        .exported_packages
        .iter()
        .map(|p| format!("    \"{}\": path.join(src, \"{}\"),\n", p.name, p.folder))
        .collect();
    format!(
        r#"import path from "node:path";
import {{ Config }} from "@remotion/cli/config";
import {{ makeBundlerOverride }} from "./src/sidecar/bundler-override";

const src = path.join(process.cwd(), "src");

Config.setRspack(true);
Config.overrideRspackConfig(
  makeBundlerOverride(path.join(src, "video"), {{
{packages}  }}),
);
"#
    )
}

fn tsconfig() -> Value {
    let paths: Map<String, Value> = LAYOUT
        .exported_packages
        .iter()
        .map(|p| {
            (
                format!("{}/*", p.name),
                json!([format!("./src/{}/*", p.folder)]),
            )
        })
        .collect();
    json!({
        "compilerOptions": {
            "target": "ESNext",
            "module": "preserve",
            "moduleResolution": "bundler",
            "jsx": "react-jsx",
            "strict": true,
            "skipLibCheck": true,
            "resolveJsonModule": true,
            "allowImportingTsExtensions": true,
            "noEmit": true,
            "paths": paths
        },
        "include": ["src"]
    })
}

fn readme(name: &str) -> String {
    format!(
        "# {name}\n\nExported from Opsis. Edit `src/timeline.json` or the components in `src/video`, then:\n\n```sh\nbun install\nbun run studio\n```\n"
    )
}

fn is_remotion(name: &str) -> bool {
    name == "remotion" || name.starts_with("@remotion/")
}

fn package_of(specifier: &str) -> Option<String> {
    if LOCAL_SPECIFIER.is_match(specifier) {
        return None;
    }
    let parts: Vec<&str> = specifier.split('/').collect();
    let take = if specifier.starts_with('@') { 2 } else { 1 };
    Some(parts[..take.min(parts.len())].join("/"))
}

fn imports_in_copy(imports: IndexMap<String, String>) -> IndexMap<String, String> {
    imports
        .into_iter()
        .map(
            |(key, target)| match target.strip_prefix(PACKAGE_SOURCE_PREFIX) {
                Some(rest) => (key, format!("./{rest}")),
                None => (key, target),
            },
        )
        .collect()
}

fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    for entry in walkdir::WalkDir::new(from) {
        let entry = entry?;
        let inside = entry
            .path()
            .strip_prefix(from)
            .map_err(std::io::Error::other)?;
        let target = if inside.as_os_str().is_empty() {
            to.to_path_buf()
        } else {
            to.join(inside)
        };
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn copy_sources(
    ctx: &ExecutorContext,
    sources: &ExportSources,
    dir: &Path,
) -> Result<(), ExecError> {
    let video = ExportedSource {
        from: LAYOUT.video_source.clone(),
        to: "video".into(),
    };
    if !sources.path(&video).exists() {
        return Err(ExecError::new(CANT_EXPORT_CODE));
    }
    for source in &LAYOUT.exported_sources {
        copy_tree(&sources.path(source), &dir.join("src").join(&source.to))?;
    }
    copy_tree(&ctx.env.public_dir, &dir.join("public"))?;
    for package in &LAYOUT.exported_packages {
        let manifest: Option<Imports> = read_json(&sources.path(&package.manifest))?;
        if let Some(imports) = manifest.and_then(|m| m.imports) {
            write_json(
                &dir.join("src").join(&package.folder).join("package.json"),
                &json!({ "type": "module", "imports": imports_in_copy(imports) }),
            )?;
        }
    }
    Ok(())
}

fn imported_packages(src: &Path) -> std::io::Result<IndexSet<String>> {
    let mut names = IndexSet::new();
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        if !entry.file_type().is_file()
            || !SOURCE_FILE.is_match(&entry.file_name().to_string_lossy())
        {
            continue;
        }
        let text = std::fs::read_to_string(entry.path())?;
        names.extend(
            IMPORT_SPECIFIER
                .captures_iter(&text)
                .filter_map(|found| package_of(&found[1])),
        );
    }
    Ok(names)
}

fn item_props(item: &TimelineItem) -> Value {
    match item {
        TimelineItem::Group { props, .. } => serde_json::to_value(props),
        TimelineItem::Recording { props, .. } => serde_json::to_value(props),
        TimelineItem::Element { props, .. } => serde_json::to_value(props),
    }
    .expect("props serialize")
}

fn project_files(spec: &TimelineSpec) -> IndexSet<String> {
    let mut values = vec![
        json!(spec.audio.iter().map(|a| a.src.clone()).collect::<Vec<_>>()),
        json!(spec.mix.as_ref().map(|m| m.src.clone())),
    ];
    walk_items(&spec.tracks, &mut |item| values.push(item_props(item)));
    let mut refs = Vec::new();
    for value in &values {
        strings_in(value, &is_project_ref, &mut refs);
    }
    refs.into_iter().filter_map(project_rel_path).collect()
}

fn check_project_files(ctx: &ExecutorContext, files: &IndexSet<String>) -> Result<(), ExecError> {
    match files.iter().find(|rel| !ctx.project_dir.join(rel).exists()) {
        Some(rel) => Err(ExecError::new(format!(
            "“{rel}” is missing from the project. Choose the file again, or remove it."
        ))),
        None => Ok(()),
    }
}

fn copy_project_files(
    ctx: &ExecutorContext,
    files: &IndexSet<String>,
    dir: &Path,
) -> std::io::Result<()> {
    for rel in files {
        let target = dir.join("public").join("project").join(rel);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(ctx.project_dir.join(rel), target)?;
    }
    Ok(())
}

fn published(
    deps: Option<&IndexMap<String, String>>,
) -> impl Iterator<Item = (String, String)> + '_ {
    deps.into_iter()
        .flatten()
        .filter(|(_, version)| !version.starts_with("workspace:"))
        .map(|(n, v)| (n.clone(), v.clone()))
}

fn package_json_for(sources: &ExportSources, name: &str, dir: &Path) -> Result<Value, ExecError> {
    let manifests: Vec<Manifest> = LAYOUT
        .dependency_manifests
        .iter()
        .map(|source| read_json(&sources.path(source)))
        .collect::<Result<Option<Vec<_>>, _>>()?
        .ok_or_else(|| ExecError::new(CANT_EXPORT_CODE))?;
    let dependencies: IndexMap<String, String> = manifests
        .iter()
        .flat_map(|m| published(m.dependencies.as_ref()))
        .collect();
    let mut versions: IndexMap<String, String> = manifests
        .iter()
        .flat_map(|m| published(m.dev_dependencies.as_ref()))
        .collect();
    versions.extend(dependencies.clone());
    let pick = |names: &mut dyn Iterator<Item = &str>| -> Map<String, Value> {
        names
            .filter_map(|n| versions.get(n).map(|v| (n.to_owned(), json!(v))))
            .collect()
    };
    let imported = imported_packages(&dir.join("src"))?;
    let runtime: Vec<&str> = dependencies
        .keys()
        .map(String::as_str)
        .filter(|n| {
            is_remotion(n)
                || EXPORT_PEERS.contains(n)
                || (imported.contains(*n) && !EXPORT_DEV_DEPENDENCIES.contains(n))
        })
        .collect();
    Ok(json!({
        "name": name,
        "private": true,
        "type": "module",
        "scripts": {
            "studio": "remotion studio src/index.ts",
            "render": "remotion render src/index.ts DemoTimeline out/video.mp4"
        },
        "dependencies": pick(&mut runtime.into_iter()),
        "devDependencies": pick(&mut EXPORT_DEV_DEPENDENCIES.into_iter())
    }))
}

async fn remove_dir(dir: &Path) -> std::io::Result<()> {
    match tokio::fs::remove_dir_all(dir).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        done => done,
    }
}

pub async fn export_code(
    ctx: Arc<ExecutorContext>,
    _: PortValues,
    engine: Arc<RenderEngine>,
) -> Result<ExecutorOutput, ExecError> {
    let p: ExportCode = ctx.params()?;
    let spec = mixed(&ctx, &*engine.tools, spec_for(&ctx)?, EXPORT_MIX_STAGE).await?;
    let name = slug_name(&p.folder_name, EXPORT_FOLDER_FALLBACK);
    let dir = ctx.out_dir.join(&name);
    let files = project_files(&spec);
    check_project_files(&ctx, &files)?;
    remove_dir(&dir).await?;
    copy_sources(&ctx, &engine.sources, &dir)?;
    copy_project_files(&ctx, &files, &dir)?;
    write_json(
        &dir.join("package.json"),
        &package_json_for(&engine.sources, &name, &dir)?,
    )?;
    let timeline = TimelineSpec {
        assets: Some(SpecAssets {
            project_base: Some(STATIC_PROJECT_BASE.to_owned()),
            versions: None,
        }),
        ..spec
    };
    write_json(&dir.join("src").join("timeline.json"), &timeline)?;
    std::fs::write(dir.join("src").join("index.ts"), INDEX_TS)?;
    std::fs::write(dir.join("src").join("export-root.tsx"), EXPORT_ROOT_TSX)?;
    std::fs::write(dir.join("remotion.config.ts"), remotion_config_ts())?;
    write_json(&dir.join("tsconfig.json"), &tsconfig())?;
    std::fs::write(dir.join("README.md"), readme(&p.folder_name))?;
    let outside = || ExecError::new(format!("{} isn't in the project.", dir.display()));
    let folder = SrcValue {
        src: Some(to_project_ref(&ctx.project_dir, &dir).ok_or_else(outside)?),
    };
    let artifact = Artifact {
        kind: ArtifactKind::File,
        path: to_posix_rel(&ctx.project_dir, &dir).ok_or_else(outside)?,
        label: Some("Video project code".into()),
        bytes: None,
        stamp: None,
    };
    Ok(ExecutorOutput {
        outputs: [("folder".to_owned(), CompileValue::Folder(folder))]
            .into_iter()
            .collect(),
        artifacts: vec![artifact],
        trace_id: None,
    })
}
