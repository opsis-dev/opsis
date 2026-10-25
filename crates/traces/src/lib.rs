use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use opsis_engine::files::{Prune, atomic_write, none_if_missing, prune_dir, read_dir_if_present};
use opsis_engine::hash::sha256_hex;
use opsis_types::actions::Viewport;
use opsis_types::project::TRACES_DIR;
use opsis_types::settings::{TraceRetention, TracesSection};
use opsis_types::trace::{
    TraceAction, TraceConsole, TraceError, TraceFrame, TraceHeader, TraceModel, TraceRequest,
};
use serde::{Deserialize, Serialize};

const MANIFEST_SUFFIX: &str = ".json.zst";
const ZSTD_LEVEL: i32 = 19;
const FRAME_EXT: &str = "jpeg";
const PROJECT_SCHEME: &str = "project:";

#[derive(Debug, thiserror::Error)]
pub enum TraceStoreError {
    #[error("Invalid trace id")]
    InvalidId,
    #[error("This trace can't be read.")]
    Unreadable,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub struct TakeFrame {
    pub jpeg: Vec<u8>,
    pub time: f64,
    pub width: u32,
    pub height: u32,
}

#[derive(Default)]
pub struct TraceTake {
    pub viewport: Option<Viewport>,
    pub actions: Vec<TraceAction>,
    pub frames: Vec<TakeFrame>,
    pub console: Vec<TraceConsole>,
    pub errors: Vec<TraceError>,
    pub network: Vec<TraceRequest>,
}

pub struct TraceMeta<'a> {
    pub node_id: &'a str,
    pub label: &'a str,
    pub ok: bool,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredFrame {
    blob: String,
    time: f64,
    width: u32,
    height: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    #[serde(flatten)]
    header: TraceHeader,
    viewport: Option<Viewport>,
    actions: Vec<TraceAction>,
    frames: Vec<StoredFrame>,
    console: Vec<TraceConsole>,
    errors: Vec<TraceError>,
    network: Vec<TraceRequest>,
}

fn traces_root(project_dir: &Path) -> PathBuf {
    project_dir.join(TRACES_DIR)
}

fn runs_dir(root: &Path) -> PathBuf {
    root.join("runs")
}

fn manifest_path(root: &Path, id: &str) -> PathBuf {
    runs_dir(root).join(format!("{id}{MANIFEST_SUFFIX}"))
}

fn blob_rel(name: &str) -> String {
    format!("blobs/{}/{name}", &name[..2])
}

fn store_blob(root: &Path, data: &[u8], ext: &str) -> std::io::Result<String> {
    let name = format!("{}.{ext}", sha256_hex(data, 64));
    let file = root.join(blob_rel(&name));
    if !file.exists() {
        std::fs::create_dir_all(file.parent().unwrap_or(root))?;
        atomic_write(&file, data)?;
    }
    Ok(name)
}

fn trace_id(node_id: &str, created_at: i64) -> String {
    let name = match slug::slugify(node_id) {
        name if name.is_empty() => "node".to_owned(),
        name => name,
    };
    format!("{name}-{created_at}")
}

fn by_time<T>(mut list: Vec<T>, time: impl Fn(&T) -> f64) -> Vec<T> {
    list.sort_by(|a, b| time(a).total_cmp(&time(b)));
    list
}

pub fn store_trace(
    project_dir: &Path,
    meta: TraceMeta<'_>,
    take: TraceTake,
) -> std::io::Result<TraceHeader> {
    let root = traces_root(project_dir);
    let frames = take
        .frames
        .iter()
        .map(|f| {
            let blob = store_blob(&root, &f.jpeg, FRAME_EXT)?;
            Ok(StoredFrame {
                blob,
                time: f.time,
                width: f.width,
                height: f.height,
            })
        })
        .collect::<std::io::Result<Vec<_>>>()?;
    let header = TraceHeader {
        id: trace_id(meta.node_id, meta.created_at),
        node_id: meta.node_id.to_owned(),
        label: meta.label.to_owned(),
        ok: meta.ok,
        created_at: meta.created_at,
    };
    let manifest = Manifest {
        header: header.clone(),
        viewport: take.viewport,
        actions: by_time(take.actions, |a| a.start),
        frames: by_time(frames, |f| f.time),
        console: by_time(take.console, |c| c.time),
        errors: by_time(take.errors, |e| e.time),
        network: by_time(take.network, |r| r.start),
    };
    let json = serde_json::to_string(&manifest)
        .map_err(std::io::Error::other)?
        .into_bytes();
    std::fs::create_dir_all(runs_dir(&root))?;
    atomic_write(
        &manifest_path(&root, &header.id),
        &zstd::encode_all(json.as_slice(), ZSTD_LEVEL)?,
    )?;
    Ok(header)
}

fn parse_manifest(data: &[u8]) -> Option<Manifest> {
    serde_json::from_slice(&zstd::decode_all(data).ok()?).ok()
}

fn plain_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn time_bounds(actions: &[TraceAction], frames: &[TraceFrame]) -> (f64, f64) {
    let times = actions
        .iter()
        .flat_map(|a| [a.start, a.end])
        .chain(frames.iter().map(|f| f.time))
        .filter(|t| *t > 0.0);
    times
        .fold(None, |bounds, t| match bounds {
            None => Some((t, t)),
            Some((start, end)) => Some((f64::min(start, t), f64::max(end, t))),
        })
        .unwrap_or((0.0, 0.0))
}

pub fn read_trace(project_dir: &Path, id: &str) -> Result<TraceModel, TraceStoreError> {
    if !plain_id(id) {
        return Err(TraceStoreError::InvalidId);
    }
    let data = none_if_missing(std::fs::read(manifest_path(&traces_root(project_dir), id)))?;
    let manifest = data
        .as_deref()
        .and_then(parse_manifest)
        .ok_or(TraceStoreError::Unreadable)?;
    let frames: Vec<TraceFrame> = manifest
        .frames
        .into_iter()
        .map(|f| TraceFrame {
            src: format!("{PROJECT_SCHEME}{TRACES_DIR}/{}", blob_rel(&f.blob)),
            time: f.time,
            width: f.width,
            height: f.height,
        })
        .collect();
    let (start, end) = time_bounds(&manifest.actions, &frames);
    Ok(TraceModel {
        header: manifest.header,
        start,
        end,
        viewport: manifest.viewport,
        actions: manifest.actions,
        frames,
        console: manifest.console,
        errors: manifest.errors,
        network: manifest.network,
    })
}

struct Runs {
    manifests: Vec<Manifest>,
    broken: Vec<PathBuf>,
}

fn read_runs(root: &Path) -> std::io::Result<Runs> {
    let dir = runs_dir(root);
    let mut runs = Runs {
        manifests: Vec::new(),
        broken: Vec::new(),
    };
    for name in read_dir_if_present(&dir)?
        .into_iter()
        .filter(|n| n.ends_with(MANIFEST_SUFFIX))
    {
        let file = dir.join(name);
        let Some(data) = none_if_missing(std::fs::read(&file))? else {
            continue;
        };
        match parse_manifest(&data) {
            Some(manifest) => runs.manifests.push(manifest),
            None => runs.broken.push(file),
        }
    }
    Ok(runs)
}

fn kept_ids(manifests: &[Manifest], policy: &TracesSection) -> HashSet<String> {
    let mut by_node: HashMap<&str, Vec<&TraceHeader>> = HashMap::new();
    for header in manifests.iter().map(|m| &m.header) {
        by_node.entry(&header.node_id).or_default().push(header);
    }
    by_node
        .into_values()
        .flat_map(|mut list| {
            list.sort_by_key(|h| std::cmp::Reverse(h.created_at));
            list.into_iter()
                .enumerate()
                .filter(|(i, h)| match policy.retention {
                    TraceRetention::Always => true,
                    TraceRetention::Failures => !h.ok,
                    TraceRetention::LastN => *i < policy.keep_last as usize,
                    TraceRetention::Off => false,
                })
        })
        .map(|(_, h)| h.id.clone())
        .collect()
}

pub fn apply_retention(
    project_dir: &Path,
    policy: &TracesSection,
) -> std::io::Result<HashSet<String>> {
    let root = traces_root(project_dir);
    let runs = read_runs(&root)?;
    let keep = kept_ids(&runs.manifests, policy);
    for manifest in runs
        .manifests
        .iter()
        .filter(|m| !keep.contains(&m.header.id))
    {
        none_if_missing(std::fs::remove_file(manifest_path(
            &root,
            &manifest.header.id,
        )))?;
    }
    for file in &runs.broken {
        none_if_missing(std::fs::remove_file(file))?;
    }
    let live: HashSet<&str> = runs
        .manifests
        .iter()
        .filter(|m| keep.contains(&m.header.id))
        .flat_map(|m| m.frames.iter().map(|f| f.blob.as_str()))
        .collect();
    let blobs = root.join("blobs");
    for shard in read_dir_if_present(&blobs)? {
        prune_dir(
            &blobs.join(shard),
            Prune {
                keep: Some(&|name: &str| live.contains(name)),
                ..Prune::default()
            },
        )?;
    }
    Ok(keep)
}

#[cfg(test)]
mod tests {
    use opsis_types::trace::ConsoleLevel;

    use super::*;

    const START: i64 = 1_767_225_600_000;

    fn action(id: &str, node_id: Option<&str>, start: f64, end: f64) -> TraceAction {
        TraceAction {
            id: id.into(),
            parent_id: None,
            group: node_id.is_some(),
            node_id: node_id.map(str::to_owned),
            title: id.into(),
            target: None,
            start,
            end,
            error: None,
            logs: Vec::new(),
            point: None,
            bounds: None,
        }
    }

    fn frame(byte: u8, time: f64) -> TakeFrame {
        TakeFrame {
            jpeg: vec![255, 216, 255, byte],
            time,
            width: 1280,
            height: 720,
        }
    }

    fn take(marker: u8) -> TraceTake {
        TraceTake {
            viewport: Some(Viewport {
                width: 1280.0,
                height: 720.0,
            }),
            actions: vec![
                action("navigate", None, 110.0, 400.0),
                action("open", Some("nav"), 100.0, 410.0),
            ],
            frames: vec![frame(2, 800.0), frame(1, 300.0), frame(marker, 500.0)],
            console: vec![TraceConsole {
                level: ConsoleLevel::Warning,
                text: "slow".into(),
                time: 200.0,
                url: None,
            }],
            ..TraceTake::default()
        }
    }

    fn meta(node_id: &str, ok: bool, at: i64) -> TraceMeta<'_> {
        TraceMeta {
            node_id,
            label: "Record",
            ok,
            created_at: START + at,
        }
    }

    fn blobs(dir: &Path) -> Vec<String> {
        let root = traces_root(dir).join("blobs");
        let mut names: Vec<String> = read_dir_if_present(&root)
            .unwrap()
            .into_iter()
            .flat_map(|shard| read_dir_if_present(&root.join(shard)).unwrap())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn reads_a_stored_trace_with_its_nodes_and_its_frames_from_the_blob_store() {
        let dir = tempfile::tempdir().unwrap();
        let header = store_trace(dir.path(), meta("rec", false, 0), take(3)).unwrap();
        let trace = read_trace(dir.path(), &header.id).unwrap();
        assert_eq!(
            (trace.header, trace.start, trace.end),
            (header, 100.0, 800.0)
        );
        let nodes: Vec<_> = trace
            .actions
            .iter()
            .map(|a| (a.id.as_str(), a.node_id.as_deref()))
            .collect();
        assert_eq!(nodes, [("open", Some("nav")), ("navigate", None)]);
        assert_eq!(
            trace.frames.iter().map(|f| f.time).collect::<Vec<_>>(),
            [300.0, 500.0, 800.0]
        );
        for f in &trace.frames {
            let rel = f.src.strip_prefix("project:.opsis/traces/blobs/").unwrap();
            assert!(
                rel.len() == 2 + 1 + 64 + 5 && rel.ends_with(".jpeg"),
                "{}",
                f.src
            );
            assert!(
                dir.path()
                    .join(f.src.strip_prefix("project:").unwrap())
                    .exists()
            );
        }
        assert_eq!(trace.console[0].level, ConsoleLevel::Warning);
    }

    #[test]
    fn stores_identical_frames_once_across_runs() {
        let dir = tempfile::tempdir().unwrap();
        store_trace(dir.path(), meta("rec", true, 0), take(3)).unwrap();
        store_trace(dir.path(), meta("rec", true, 1_000), take(4)).unwrap();
        assert_eq!(blobs(dir.path()).len(), 4);
    }

    #[test]
    fn keeps_only_failures_when_asked_and_collects_orphaned_blobs() {
        let dir = tempfile::tempdir().unwrap();
        let ok = store_trace(dir.path(), meta("rec", true, 0), take(3)).unwrap();
        let bad = store_trace(dir.path(), meta("rec", false, 1_000), take(4)).unwrap();
        let policy = TracesSection {
            retention: TraceRetention::Failures,
            keep_last: 10,
        };
        assert_eq!(
            apply_retention(dir.path(), &policy).unwrap(),
            HashSet::from([bad.id.clone()])
        );
        assert!(matches!(
            read_trace(dir.path(), &ok.id),
            Err(TraceStoreError::Unreadable)
        ));
        assert!(!read_trace(dir.path(), &bad.id).unwrap().header.ok);
        assert_eq!(blobs(dir.path()).len(), 3);
    }

    #[test]
    fn keeps_the_newest_traces_of_each_node() {
        let dir = tempfile::tempdir().unwrap();
        let ids: Vec<String> = (0..3)
            .map(|i| {
                store_trace(dir.path(), meta("rec", true, i), take(3))
                    .unwrap()
                    .id
            })
            .collect();
        let other = store_trace(dir.path(), meta("other", true, 0), take(3)).unwrap();
        let policy = TracesSection {
            retention: TraceRetention::LastN,
            keep_last: 2,
        };
        assert_eq!(
            apply_retention(dir.path(), &policy).unwrap(),
            HashSet::from([ids[1].clone(), ids[2].clone(), other.id])
        );
        let off = TracesSection {
            retention: TraceRetention::Off,
            keep_last: 2,
        };
        assert!(apply_retention(dir.path(), &off).unwrap().is_empty());
        assert!(blobs(dir.path()).is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn keeps_every_blob_when_a_manifest_cant_be_read_for_a_moment() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let failed = store_trace(dir.path(), meta("rec", false, 0), take(3)).unwrap();
        store_trace(dir.path(), meta("rec", true, 1_000), take(4)).unwrap();
        let before = blobs(dir.path());
        let manifest = manifest_path(&traces_root(dir.path()), &failed.id);
        std::fs::set_permissions(&manifest, std::fs::Permissions::from_mode(0o000)).unwrap();
        let policy = TracesSection {
            retention: TraceRetention::Failures,
            keep_last: 10,
        };
        let result = apply_retention(dir.path(), &policy);
        std::fs::set_permissions(&manifest, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert_eq!(blobs(dir.path()), before);
    }

    #[test]
    fn removes_broken_manifests_and_still_applies_retention() {
        let dir = tempfile::tempdir().unwrap();
        let ok = store_trace(dir.path(), meta("rec", true, 0), take(3)).unwrap();
        let broken = runs_dir(&traces_root(dir.path())).join("broken-1.json.zst");
        std::fs::write(&broken, "not zstd").unwrap();
        let policy = TracesSection {
            retention: TraceRetention::Failures,
            keep_last: 10,
        };
        assert!(apply_retention(dir.path(), &policy).unwrap().is_empty());
        assert!(!broken.exists());
        assert!(matches!(
            read_trace(dir.path(), &ok.id),
            Err(TraceStoreError::Unreadable)
        ));
        assert!(blobs(dir.path()).is_empty());
    }

    #[test]
    fn reads_traces_of_nodes_whose_ids_arent_plain_names_and_refuses_ids_that_arent() {
        let dir = tempfile::tempdir().unwrap();
        let header = store_trace(dir.path(), meta("my node/1.x", true, 0), take(3)).unwrap();
        assert_eq!(
            read_trace(dir.path(), &header.id).unwrap().header.node_id,
            "my node/1.x"
        );
        assert!(matches!(
            read_trace(dir.path(), "gone-1"),
            Err(TraceStoreError::Unreadable)
        ));
        assert!(matches!(
            read_trace(dir.path(), "../../etc/passwd"),
            Err(TraceStoreError::InvalidId)
        ));
    }
}
