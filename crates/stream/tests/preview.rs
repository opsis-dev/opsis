use std::path::{Path, PathBuf};
use std::time::Duration;

use opsis_capture::browser::shell_in;
use opsis_compile::compile::compile_workflow;
use opsis_project::last_run::read_last_run;
use opsis_stream::host::ViewEnv;
use opsis_stream::preview::{PlayerState, Preview, PreviewDirs};
use opsis_stream::view::ViewSize;
use opsis_types::timeline::{PreviewQuality, TimelineSpec};
use opsis_types::workflow::Workflow;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn take_project() -> PathBuf {
    std::env::var_os("OPSIS_PREVIEW_PROJECT")
        .map(PathBuf::from)
        .expect("OPSIS_PREVIEW_PROJECT names a project that recorded the TodoMVC take")
}

fn film(project: &Path) -> TimelineSpec {
    let workflow: Workflow = serde_json::from_str(
        &std::fs::read_to_string(root().join("examples/todomvc/project.opsis.json")).unwrap(),
    )
    .unwrap();
    let results = read_last_run(project).unwrap().expect("a saved run").nodes;
    let compiled = compile_workflow(&workflow, Some(&results));
    TimelineSpec {
        quality: Some(PreviewQuality::Fast),
        ..compiled.specs.get("render").cloned().expect("the film")
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine, a built preview page and a recorded take"]
async fn previews_the_film_frame_by_frame() {
    let temp = tempfile::tempdir().unwrap();
    let engine = PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config/Opsis/engine");
    let env = ViewEnv {
        executable: shell_in(&engine).unwrap(),
        temp_dir: temp.path().to_path_buf(),
    };
    let project = take_project();
    let spec = film(&project);
    let (page, public) = (root().join("apps/preview/dist"), root().join("public"));
    let dirs = PreviewDirs {
        page: &page,
        public: &public,
        project: &project,
    };
    let preview = Preview::open(
        &env,
        &dirs,
        ViewSize {
            width: 960.0,
            height: 540.0,
            dpr: 1.0,
        },
        80,
    )
    .await
    .unwrap();
    preview.load(&spec).await.unwrap();
    let mut frames = preview.frames();
    preview.seek(300.0).await.unwrap();
    tokio::time::timeout(Duration::from_secs(10), frames.wait_for(|f| *f == 300.0))
        .await
        .unwrap()
        .unwrap();
    let still = preview.still().await.unwrap();
    let size = imagesize::blob_size(&still).unwrap();
    assert_eq!((size.width, size.height), (960, 540));
    preview.play().await.unwrap();
    preview.until(|s| s == PlayerState::Playing).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    let played = *preview.frames().borrow();
    assert!(played > 315.0, "only reached frame {played}");
    preview.close().await;
}
