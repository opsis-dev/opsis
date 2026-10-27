use opsis_engine::tools::EngineFiles;
use std::path::Path;
use std::sync::{Arc, Mutex};

use opsis_compile::registry::default_params;
use opsis_compile::vars::Vars;
use opsis_media::executors::{
    LinkedMedia, Tools, media_executors, media_has_sound, thumbnail_time,
};
use opsis_media::ffmpeg::{
    AudioInfo, FramePipe, MediaTools, ProbeResult, RunOptions, ToolError, ToolFuture, VideoInfo,
};
use opsis_runner::context::{ExecError, ExecutorContext, ExecutorOutput, ResultStore, RunEnv};
use opsis_types::settings::AppSettings;
use opsis_types::values::{ArtifactKind, PortValues};
use opsis_types::workflow::Workflow;
use pretty_assertions::assert_eq;
use serde_json::{Map, Value, json};
use tokio_util::sync::CancellationToken;

struct FakeTools {
    probe: ProbeResult,
    ran: Mutex<Vec<Vec<String>>>,
}

impl MediaTools for FakeTools {
    fn run<'a>(&'a self, args: &'a [String], _: RunOptions<'a>) -> ToolFuture<'a, ()> {
        self.ran.lock().unwrap().push(args.to_vec());
        Box::pin(async { Ok(()) })
    }

    fn probe<'a>(&'a self, _: &'a Path) -> ToolFuture<'a, ProbeResult> {
        let probe = self.probe.clone();
        Box::pin(async move { Ok(probe) })
    }

    fn pipe(&self, args: &[String], _: Option<CancellationToken>) -> Result<FramePipe, ToolError> {
        self.ran.lock().unwrap().push(args.to_vec());
        Ok(FramePipe {
            stdin: Box::pin(tokio::io::sink()),
            done: Box::pin(async { Ok(()) }),
        })
    }
}

fn film_probe(sound: Option<&str>) -> ProbeResult {
    ProbeResult {
        duration_ms: Some(4000),
        format: "mov,mp4".into(),
        video: Some(VideoInfo {
            codec: "h264".into(),
            width: 1920,
            height: 1080,
            fps: Some(30.0),
            pix_fmt: "yuv420p".into(),
        }),
        audio: sound.map(|codec| AudioInfo {
            codec: codec.into(),
            sample_rate: Some(48000),
            channels: Some(2),
        }),
        raw: json!({}),
    }
}

fn sound_probe() -> ProbeResult {
    ProbeResult {
        duration_ms: Some(4000),
        format: "wav".into(),
        video: None,
        audio: Some(AudioInfo {
            codec: "pcm_s16le".into(),
            sample_rate: Some(48000),
            channels: Some(2),
        }),
        raw: json!({}),
    }
}

fn with_defaults(kind: &str, params: Value) -> Map<String, Value> {
    let mut values = default_params(kind);
    values.extend(params.as_object().cloned().unwrap_or_default());
    values
}

struct Setup<'a> {
    kind: &'a str,
    params: Value,
    project_dir: &'a Path,
    names: &'a [(&'a str, &'a str)],
}

async fn run(
    setup: Setup<'_>,
    probe: ProbeResult,
    inputs: Value,
) -> (Result<ExecutorOutput, ExecError>, Vec<Vec<String>>) {
    let fake = Arc::new(FakeTools {
        probe,
        ran: Mutex::new(Vec::new()),
    });
    let tools: Tools = fake.clone();
    let named: Vec<Value> = setup
        .names
        .iter()
        .map(|(id, label)| json!({ "id": id, "type": "grade.lookFile", "position": { "x": 0, "y": 0 }, "label": label }))
        .collect();
    let workflow: Workflow = serde_json::from_value(
        json!({ "version": 1, "id": "x", "name": "x", "nodes": named, "edges": [] }),
    )
    .unwrap();
    let node = serde_json::from_value(json!({ "id": "n", "type": setup.kind, "position": { "x": 0, "y": 0 }, "params": with_defaults(setup.kind, setup.params) })).unwrap();
    let workflow = Arc::new(workflow);
    let ctx = ExecutorContext {
        run_id: "run".into(),
        node,
        workflow: Arc::clone(&workflow),
        env: Arc::new(RunEnv {
            app_version: "test".into(),
            engine: EngineFiles::default(),
            public_dir: setup.project_dir.join("public"),
            temp_dir: setup.project_dir.into(),
            settings: AppSettings::default(),
        }),
        project_dir: setup.project_dir.into(),
        dir: setup.project_dir.join(".opsis/cache/h"),
        rel_dir: ".opsis/cache/h".into(),
        out_dir: setup.project_dir.join(".opsis/out"),
        hash: "h".into(),
        forced: false,
        vars: Arc::new(Vars::default()),
        cancel: CancellationToken::new(),
        emit: Arc::new(|_| {}),
        store: Arc::new(ResultStore::new(workflow)),
    };
    let executors = media_executors(&tools);
    let (_, run) = executors
        .iter()
        .find(|(kind, _)| *kind == setup.kind)
        .unwrap();
    let inputs: PortValues = serde_json::from_value(inputs).unwrap();
    let result = run.run(Arc::new(ctx), inputs).await;
    let ran = fake.ran.lock().unwrap().clone();
    (result, ran)
}

fn message(result: Result<ExecutorOutput, ExecError>) -> String {
    result.err().map(|e| e.to_string()).unwrap_or_default()
}

fn film(kind: &str) -> Value {
    json!({ "type": "media", "kind": kind, "src": "project:assets/film.mp4" })
}

fn grade_of(steps: &[(&str, Value, &str)]) -> Value {
    let entries: Vec<Value> = steps.iter().map(|(kind, params, id)| json!({ "nodeId": id, "type": kind, "params": with_defaults(kind, params.clone()) })).collect();
    json!({ "type": "grade", "entries": entries })
}

fn grade_setup<'a>(dir: &'a Path, names: &'a [(&'a str, &'a str)]) -> Setup<'a> {
    Setup {
        kind: "media.applyGrade",
        params: json!({}),
        project_dir: dir,
        names,
    }
}

#[test]
fn thumbnail_takes_the_frame_at_the_chosen_time_when_the_file_is_long_enough() {
    assert_eq!(thumbnail_time(1000.0, 4000.0), 1000.0);
}

#[test]
fn thumbnail_takes_a_frame_just_before_the_end_of_a_shorter_file() {
    assert_eq!(thumbnail_time(1000.0, 500.0), 400.0);
    assert_eq!(thumbnail_time(60_000.0, 4000.0), 3900.0);
}

#[test]
fn thumbnail_takes_the_only_frame_of_a_still_image() {
    assert_eq!(thumbnail_time(1000.0, 0.0), 0.0);
}

#[tokio::test]
async fn apply_grade_asks_for_a_grade() {
    let (result, _) = run(
        grade_setup(Path::new("/p"), &[]),
        film_probe(Some("aac")),
        json!({ "media": film("video") }),
    )
    .await;
    assert_eq!(message(result), "Connect a grade.");
}

#[tokio::test]
async fn apply_grade_names_the_look_file_step_that_has_no_file() {
    let names = [("look", "Film look")];
    let inputs = json!({ "media": film("video"), "grade": grade_of(&[("grade.lookFile", json!({}), "look")]) });
    let (result, _) = run(
        grade_setup(Path::new("/p"), &names),
        film_probe(Some("aac")),
        inputs,
    )
    .await;
    assert_eq!(message(result), "Choose a look file for “Film look”.");
}

#[tokio::test]
async fn apply_grade_names_the_look_file_step_whose_file_is_gone() {
    let names = [("look", "Film look")];
    let inputs = json!({ "media": film("video"), "grade": grade_of(&[("grade.lookFile", json!({ "file": "assets/looks/gone.cube" }), "look")]) });
    let (result, _) = run(
        grade_setup(Path::new("/p"), &names),
        film_probe(Some("aac")),
        inputs,
    )
    .await;
    assert_eq!(
        message(result),
        "The look file for “Film look” isn't in the project."
    );
}

#[tokio::test]
async fn apply_grade_writes_one_pass_with_the_grade_graph_the_picture_format_and_the_sound_copied()
{
    let dir = tempfile::tempdir().unwrap();
    let inputs = json!({ "media": film("video"), "grade": grade_of(&[("grade.colour", json!({ "contrast": 1.2 }), "grade.colour")]) });
    let (result, ran) = run(
        grade_setup(dir.path(), &[]),
        film_probe(Some("aac")),
        inputs,
    )
    .await;
    assert!(result.is_ok());
    let film = dir
        .path()
        .join("assets/film.mp4")
        .to_string_lossy()
        .into_owned();
    let graded = dir
        .path()
        .join(".opsis/cache/h/graded.mp4")
        .to_string_lossy()
        .into_owned();
    let expected = [
        "-i",
        &film,
        "-filter_complex",
        "[0:v]eq=contrast=1.2,scale=trunc(iw/2)*2:trunc(ih/2)*2[out]",
        "-map",
        "[out]",
        "-map",
        "0:a?",
        "-c:v",
        "libx264",
        "-preset",
        "medium",
        "-crf",
        "18",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "copy",
        "-movflags",
        "+faststart",
        &graded,
    ];
    assert_eq!(ran[0], expected);
}

#[tokio::test]
async fn apply_grade_maps_no_sound_for_a_silent_file_and_writes_a_png_for_an_image() {
    let dir = tempfile::tempdir().unwrap();
    let grade = grade_of(&[("grade.colour", json!({ "contrast": 1.2 }), "grade.colour")]);
    let (_, silent) = run(
        grade_setup(dir.path(), &[]),
        film_probe(None),
        json!({ "media": film("video"), "grade": grade }),
    )
    .await;
    assert!(!silent[0].iter().any(|a| a == "0:a?"));
    let (_, still) = run(
        grade_setup(dir.path(), &[]),
        film_probe(Some("aac")),
        json!({ "media": film("image"), "grade": grade }),
    )
    .await;
    let args = &still[0];
    assert!(args.windows(2).any(|w| w == ["-frames:v", "1"]));
    assert!(args.last().unwrap().ends_with("graded.png"));
    assert!(!args.iter().any(|a| a == "-c:v"));
}

fn sound_setup(kind: &str, params: Value) -> Setup<'_> {
    Setup {
        kind,
        params,
        project_dir: Path::new("/p"),
        names: &[],
    }
}

fn voice() -> Value {
    json!({ "media": { "type": "media", "kind": "audio", "src": "project:assets/voice.wav", "durationMs": 4000 } })
}

#[tokio::test]
async fn changes_the_speed_of_a_sound_file_into_the_same_sound_format() {
    let (result, ran) = run(
        sound_setup("media.speed", json!({ "rate": 2 })),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(
        ran[0],
        [
            "-i",
            "/p/assets/voice.wav",
            "-af",
            "atempo=2.0000",
            "-vn",
            "-c:a",
            "pcm_s16le",
            "/p/.opsis/cache/h/speed.wav"
        ]
    );
    assert_eq!(result.unwrap().artifacts[0].kind, ArtifactKind::Audio);
}

#[tokio::test]
async fn says_a_sound_file_without_its_sound_leaves_nothing() {
    let (result, _) = run(
        sound_setup("media.speed", json!({ "keepAudio": false })),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(
        message(result),
        "This file is only sound, so turning off Keep the sound leaves nothing."
    );
}

#[tokio::test]
async fn reverses_a_sound_file_into_the_same_sound_format() {
    let (_, ran) = run(
        sound_setup("media.reverse", json!({})),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(
        ran[0],
        [
            "-i",
            "/p/assets/voice.wav",
            "-af",
            "areverse",
            "-vn",
            "-c:a",
            "pcm_s16le",
            "/p/.opsis/cache/h/reversed.wav"
        ]
    );
}

#[tokio::test]
async fn refuses_to_resize_a_sound_file() {
    let (result, _) = run(
        sound_setup("media.scalePad", json!({})),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(
        message(result),
        "Resize works on video and images, and this file is only sound."
    );
}

fn probing(audio: bool) -> FakeTools {
    let mut probe = film_probe(audio.then_some("aac"));
    probe.video = None;
    FakeTools {
        probe,
        ran: Mutex::new(Vec::new()),
    }
}

fn linked_film(has_audio: Option<bool>) -> LinkedMedia {
    let value = serde_json::from_value(json!({ "type": "media", "src": "project:out/film.mp4", "kind": "video", "hasAudio": has_audio })).unwrap();
    LinkedMedia {
        value,
        src: "project:out/film.mp4".into(),
    }
}

async fn has_sound(tools: &FakeTools, media: &LinkedMedia) -> bool {
    let project = Path::new("/project");
    let workflow = Arc::new(
        serde_json::from_value::<Workflow>(
            json!({ "version": 1, "id": "x", "name": "x", "nodes": [], "edges": [] }),
        )
        .unwrap(),
    );
    let node = serde_json::from_value(
        json!({ "id": "n", "type": "media.probe", "position": { "x": 0, "y": 0 } }),
    )
    .unwrap();
    let ctx = ExecutorContext {
        run_id: "run".into(),
        node,
        workflow: Arc::clone(&workflow),
        env: Arc::new(RunEnv {
            app_version: "test".into(),
            engine: EngineFiles::default(),
            public_dir: project.into(),
            temp_dir: project.into(),
            settings: AppSettings::default(),
        }),
        project_dir: project.into(),
        dir: project.into(),
        rel_dir: ".opsis/cache/h".into(),
        out_dir: project.into(),
        hash: "h".into(),
        forced: false,
        vars: Arc::new(Vars::default()),
        cancel: CancellationToken::new(),
        emit: Arc::new(|_| {}),
        store: Arc::new(ResultStore::new(workflow)),
    };
    media_has_sound(&ctx, tools, media).await.unwrap()
}

#[tokio::test]
async fn media_has_sound_probes_the_file_when_the_value_doesnt_say() {
    assert!(has_sound(&probing(true), &linked_film(None)).await);
    assert!(!has_sound(&probing(false), &linked_film(None)).await);
}

#[tokio::test]
async fn media_has_sound_trusts_the_value_when_it_says() {
    assert!(!has_sound(&probing(true), &linked_film(Some(false))).await);
}

#[tokio::test]
async fn fixes_clicks_and_clipping_in_one_pass_keeping_the_sound_format() {
    let (result, ran) = run(
        sound_setup("media.repair", json!({ "strength": 1 })),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(
        ran[0],
        [
            "-i",
            "/p/assets/voice.wav",
            "-af",
            "adeclick=t=1.00,adeclip=t=5.00",
            "-ar",
            "48000",
            "-c:a",
            "pcm_s16le",
            "/p/.opsis/cache/h/repaired.wav"
        ]
    );
    assert_eq!(result.unwrap().artifacts[0].kind, ArtifactKind::Audio);
    let (_, ran) = run(
        sound_setup("media.repair", json!({ "clipping": false })),
        film_probe(Some("aac")),
        json!({ "media": film("video") }),
    )
    .await;
    let args = ran[0].join(" ");
    assert!(args.contains("-af adeclick=t=2.50 "), "{args}");
    assert!(args.contains("-c:v copy"), "the picture is copied: {args}");
    let (result, _) = run(
        sound_setup(
            "media.repair",
            json!({ "clicks": false, "clipping": false }),
        ),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(message(result), "Turn on clicks or clipping to fix.");
}

#[tokio::test]
async fn notches_the_chosen_hum_and_its_harmonics() {
    let (_, ran) = run(
        sound_setup(
            "media.hum",
            json!({ "mains": "sixty", "harmonics": 2, "width": 3 }),
        ),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(ran.len(), 1, "a chosen hum needs no listening");
    assert_eq!(
        ran[0][3],
        "equalizer=f=60:t=h:w=3:g=-40,equalizer=f=120:t=h:w=3:g=-40,equalizer=f=180:t=h:w=3:g=-40"
    );
    let (_, ran) = run(sound_setup("media.hum", json!({})), sound_probe(), voice()).await;
    assert_eq!(
        ran.len(),
        3,
        "finding the hum listens at 50 and 60 Hz first"
    );
    assert_eq!(
        ran[0][3],
        "bandpass=f=50:t=h:w=4,astats=measure_perchannel=none:measure_overall=RMS_level"
    );
    assert!(ran[1][3].starts_with("bandpass=f=60:"));
    assert!(
        ran[2][3].starts_with("equalizer=f=50:"),
        "50 Hz when neither is heard"
    );
}

#[tokio::test]
async fn draws_a_sound_as_one_picture_of_the_chosen_size_on_its_background() {
    let (result, ran) = run(
        sound_setup(
            "media.soundPicture",
            json!({ "width": 800, "height": 200, "color": "#ff0000", "background": "#000000" }),
        ),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(
        ran[0],
        [
            "-i",
            "/p/assets/voice.wav",
            "-filter_complex",
            "[0:a]showwavespic=s=800x200:split_channels=0:colors=#ff0000,format=rgba[drawn];color=c=#000000:s=800x200,format=rgba[ground];[ground][drawn]overlay=format=auto,format=rgba[out]",
            "-map",
            "[out]",
            "-frames:v",
            "1",
            "/p/.opsis/cache/h/picture.png"
        ]
    );
    let out = result.unwrap();
    assert_eq!(out.artifacts[0].kind, ArtifactKind::Image);
    let (_, ran) = run(
        sound_setup(
            "media.soundPicture",
            json!({ "kind": "spectrum", "colours": "fire" }),
        ),
        sound_probe(),
        voice(),
    )
    .await;
    assert!(
        ran[0][3]
            .starts_with("[0:a]showspectrumpic=s=1920x400:legend=0:color=fire,scale=1920x400,")
    );
}

#[tokio::test]
async fn shortens_pauses_after_listening_for_them_and_refuses_videos() {
    let (result, ran) = run(
        sound_setup("media.shortenPauses", json!({})),
        sound_probe(),
        voice(),
    )
    .await;
    assert!(result.is_ok());
    assert_eq!(ran[0][3], "silencedetect=n=-40dB:d=0.7");
    assert_eq!(
        ran[1][..6],
        [
            "-i",
            "/p/assets/voice.wav",
            "-filter_complex",
            "[0:a]atrim=start=0.000:end=4.000,asetpts=PTS-STARTPTS[p0];[p0]anull[out]",
            "-map",
            "[out]"
        ]
    );
    let (result, _) = run(
        sound_setup("media.shortenPauses", json!({})),
        film_probe(Some("aac")),
        json!({ "media": film("video") }),
    )
    .await;
    assert!(message(result).starts_with("Shorten pauses works on sound files"));
}

async fn made_sound(kind: &str, params: Value) -> (ExecutorOutput, Vec<i16>, u32) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".opsis/cache/h")).unwrap();
    let setup = Setup {
        kind,
        params,
        project_dir: dir.path(),
        names: &[],
    };
    let (result, ran) = run(setup, sound_probe(), json!({})).await;
    assert!(ran.is_empty());
    let out = result.unwrap();
    let mut reader = hound::WavReader::open(dir.path().join(&out.artifacts[0].path)).unwrap();
    let rate = reader.spec().sample_rate;
    let samples = reader.samples::<i16>().map(Result::unwrap).collect();
    (out, samples, rate)
}

fn level_db(samples: &[i16]) -> f64 {
    let mean = samples
        .iter()
        .map(|s| (f64::from(*s) / f64::from(i16::MAX)).powi(2))
        .sum::<f64>()
        / samples.len() as f64;
    10.0 * mean.log10()
}

#[tokio::test]
async fn makes_a_ten_second_loop_of_room_tone_at_its_level() {
    let (out, samples, rate) =
        made_sound("audio.roomTone", json!({ "colour": "brown", "level": -40 })).await;
    assert_eq!(rate, 48000);
    assert_eq!(samples.len(), 2 * 480_000);
    assert!(
        (level_db(&samples) + 40.0).abs() < 0.1,
        "{}",
        level_db(&samples)
    );
    let media = &out.outputs["audio"];
    assert_eq!(
        serde_json::to_value(media).unwrap(),
        json!({ "type": "media", "kind": "audio", "src": "project:.opsis/cache/h/room-tone.wav", "durationMs": 10000, "hasAudio": true })
    );
    assert_eq!(out.artifacts[0].kind, ArtifactKind::Audio);
}
