use std::path::Path;
use std::sync::{Arc, Mutex};

use opsis_compile::registry::default_params;
use opsis_compile::vars::Vars;
use opsis_engine::tools::EngineFiles;
use opsis_media::executors::{
    FileCheckError, LinkedMedia, Tools, check_media_file, media_executors, media_has_sound,
    thumbnail_time,
};
use opsis_media::ffmpeg::{FramePipe, MediaTools, RunOptions, ToolError, ToolFuture};
use opsis_probe::{AudioStream, Probe, VideoStream};
use opsis_runner::context::{ExecError, ExecutorContext, ExecutorOutput, ResultStore, RunEnv};
use opsis_types::numbers;
use opsis_types::settings::AppSettings;
use opsis_types::values::{ArtifactKind, Measured, MediaKind, PortValues};
use opsis_types::workflow::Workflow;
use pretty_assertions::assert_eq;
use serde_json::{Map, Value, json};
use tokio_util::sync::CancellationToken;

struct FakeTools {
    probe: Probe,
    ran: Mutex<Vec<Vec<String>>>,
}

impl MediaTools for FakeTools {
    fn run<'a>(&'a self, args: &'a [String], _: RunOptions<'a>) -> ToolFuture<'a, ()> {
        self.ran.lock().unwrap().push(args.to_vec());
        Box::pin(async { Ok(()) })
    }

    fn probe<'a>(&'a self, _: &'a Path) -> ToolFuture<'a, Probe> {
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

fn film_probe(sound: Option<&str>) -> Probe {
    Probe {
        duration_ms: Some(4000),
        format: "mov,mp4".into(),
        video: Some(VideoStream {
            codec: "h264".into(),
            width: 1920,
            height: 1080,
            fps: Some(30.0),
            pix_fmt: "yuv420p".into(),
        }),
        audio: sound.map(|codec| AudioStream {
            codec: codec.into(),
            sample_rate: Some(48000),
            channels: Some(2),
        }),
    }
}

fn sound_probe() -> Probe {
    Probe {
        duration_ms: Some(4000),
        format: "wav".into(),
        video: None,
        audio: Some(AudioStream {
            codec: "pcm_s16le".into(),
            sample_rate: Some(48000),
            channels: Some(2),
        }),
    }
}

fn with_defaults(kind: &str, params: Value) -> Map<String, Value> {
    let mut values = default_params(kind);
    if let Value::Object(params) = params {
        values.extend(params);
    }
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
    probe: Probe,
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
    result.err().map_or_default(|e| e.to_string())
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
    let at = thumbnail_time(1000.0, 4000.0);
    assert!(numbers::exact_eq(at, 1000.0), "{at}");
}

#[test]
fn thumbnail_takes_a_frame_just_before_the_end_of_a_shorter_file() {
    let short = thumbnail_time(1000.0, 500.0);
    assert!(numbers::exact_eq(short, 400.0), "{short}");
    let long = thumbnail_time(60_000.0, 4000.0);
    assert!(numbers::exact_eq(long, 3900.0), "{long}");
}

#[test]
fn thumbnail_takes_the_only_frame_of_a_still_image() {
    let still = thumbnail_time(1000.0, 0.0);
    assert!(numbers::exact_eq(still, 0.0), "{still}");
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
        json!({ "id": "n", "type": "media.trim", "position": { "x": 0, "y": 0 } }),
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
    assert_eq!(ran, [] as [Vec<String>; 0]);
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
        / numbers::float(samples.len());
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

#[tokio::test]
async fn makes_a_countdown_of_beeps_then_a_long_one() {
    let (out, samples, _) = made_sound(
        "audio.tones",
        json!({ "kind": "countdown", "count": 2, "spacingMs": 500, "lengthMs": 100 }),
    )
    .await;
    assert_eq!(samples.len(), 2 * (48 * 1300));
    assert_eq!(
        serde_json::to_value(&out.outputs["audio"]).unwrap()["durationMs"],
        1300
    );
    let (_, single, _) =
        made_sound("audio.tones", json!({ "kind": "tone", "lengthMs": 2000 })).await;
    assert_eq!(single.len(), 2 * 96_000);
}

#[tokio::test]
async fn cleans_speech_with_the_spectral_way_in_one_pass_between_two_listens() {
    let (result, ran) = run(
        sound_setup(
            "media.cleanSpeech",
            json!({ "method": "spectral", "noise": 0.5 }),
        ),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(result.unwrap().measured, []);
    assert_eq!(ran.len(), 3);
    assert_eq!(
        ran[0][2..6],
        [
            "-map",
            "0:a:0",
            "-af",
            "aresample=48000,asetnsamples=n=960,astats=metadata=1:reset=1:measure_perchannel=none:measure_overall=RMS_level,ametadata=mode=print:key=lavfi.astats.Overall.RMS_level"
        ]
    );
    assert_eq!(
        ran[1],
        [
            "-i",
            "/p/assets/voice.wav",
            "-af",
            "afftdn=nr=20:nf=-50,highpass=f=80,deesser=i=0.5,speechnorm=e=6.31:r=0.00001:l=1:t=0.01",
            "-ar",
            "48000",
            "-c:a",
            "pcm_s16le",
            "/p/.opsis/cache/h/speech.wav"
        ]
    );
    assert_eq!(ran[2][1], "/p/.opsis/cache/h/speech.wav");
    let (_, ran) = run(
        sound_setup(
            "media.cleanSpeech",
            json!({ "method": "spectral", "noise": 0, "softenS": 0, "evenLevel": false }),
        ),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(ran[1][3], "highpass=f=80");
}

#[tokio::test]
async fn cleans_speech_the_learned_way_and_puts_the_cleaned_voice_back_under_the_picture() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join(".opsis/cache/h");
    std::fs::create_dir_all(&cache).unwrap();
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(cache.join("voice-in.wav"), spec).unwrap();
    for i in 0..4800 {
        writer
            .write_sample(i16::try_from((i % 97) * 50).unwrap())
            .unwrap();
    }
    writer.finalize().unwrap();
    let setup = Setup {
        kind: "media.cleanSpeech",
        params: json!({}),
        project_dir: dir.path(),
        names: &[],
    };
    let (result, ran) = run(
        setup,
        film_probe(Some("aac")),
        json!({ "media": film("video") }),
    )
    .await;
    result.unwrap();
    let at = |path: &str| dir.path().join(path).to_string_lossy().into_owned();
    assert_eq!(
        ran[1],
        [
            "-i",
            &at("assets/film.mp4"),
            "-map",
            "0:a:0",
            "-ac",
            "1",
            "-ar",
            "48000",
            "-c:a",
            "pcm_s16le",
            &at(".opsis/cache/h/voice-in.wav")
        ]
    );
    assert_eq!(
        ran[2][..12],
        [
            "-i",
            &at("assets/film.mp4"),
            "-i",
            &at(".opsis/cache/h/voice.wav"),
            "-map",
            "0:v?",
            "-map",
            "1:a",
            "-ac",
            "2",
            "-af",
            "highpass=f=80,deesser=i=0.5,speechnorm=e=6.31:r=0.00001:l=1:t=0.01"
        ]
    );
    assert!(ran[2].windows(2).any(|w| w == ["-c:v", "copy"]));
    assert!(!cache.join("voice-in.wav").exists());
    assert!(!cache.join("voice.wav").exists());
}

#[tokio::test]
async fn checks_a_media_file_for_its_kind_size_and_frame_rate_or_says_it_is_gone() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("clip.mp4");
    std::fs::write(&file, vec![0u8; 3000]).unwrap();
    let tools = FakeTools {
        probe: film_probe(None),
        ran: Mutex::new(Vec::new()),
    };
    let checked = check_media_file(&tools, &file, "project:assets/clip.mp4")
        .await
        .unwrap();
    assert_eq!(
        (
            checked.media.kind,
            checked.media.has_audio,
            checked.media.duration_ms,
            checked.measured
        ),
        (
            MediaKind::Video,
            Some(false),
            Some(4000),
            Measured::File {
                bytes: 3000,
                fps: Some(30.0)
            }
        )
    );
    let quiet = FakeTools {
        probe: probing(true).probe,
        ran: Mutex::new(Vec::new()),
    };
    let sound = check_media_file(&quiet, &file, "project:assets/clip.mp4")
        .await
        .unwrap();
    assert_eq!(
        (sound.media.kind, sound.measured),
        (
            MediaKind::Audio,
            Measured::File {
                bytes: 3000,
                fps: None
            }
        )
    );
    let gone = check_media_file(
        &tools,
        &dir.path().join("gone.mp4"),
        "project:assets/gone.mp4",
    )
    .await
    .err()
    .unwrap();
    assert!(matches!(gone, FileCheckError::Missing), "{gone:?}");
    let folder = check_media_file(&tools, dir.path(), "project:assets")
        .await
        .err()
        .unwrap();
    assert_eq!(folder.to_string(), "This is a folder, not a file.");
}

#[cfg(unix)]
#[tokio::test]
async fn says_a_file_it_cannot_open_without_naming_its_path() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("locked.mp4");
    std::fs::write(&file, vec![0u8; 10]).unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o000)).unwrap();
    let tools = FakeTools {
        probe: film_probe(None),
        ran: Mutex::new(Vec::new()),
    };
    let locked = check_media_file(&tools, &file, "project:assets/locked.mp4")
        .await
        .err()
        .unwrap();
    assert_eq!(
        locked.to_string(),
        "Can't open this file: permission denied."
    );
    assert!(matches!(
        locked,
        FileCheckError::Unopenable(std::io::ErrorKind::PermissionDenied)
    ));
}

#[tokio::test]
async fn changes_pitch_in_one_pass_keeping_the_voice_character_unless_asked() {
    let (result, ran) = run(
        sound_setup("media.pitch", json!({})),
        sound_probe(),
        voice(),
    )
    .await;
    assert!(result.is_ok());
    assert_eq!(
        ran[0][..4],
        [
            "-i",
            "/p/assets/voice.wav",
            "-af",
            "apad=pad_dur=1,rubberband=pitch=1.189207:formant=preserved:pitchq=quality:transients=mixed,atrim=end=4.000"
        ]
    );
    let (_, ran) = run(
        sound_setup(
            "media.pitch",
            json!({ "semitones": -12, "keepCharacter": false }),
        ),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(
        ran[0][3],
        "apad=pad_dur=1,rubberband=pitch=0.5:formant=shifted:pitchq=quality:transients=mixed,atrim=end=4.000"
    );
}

#[tokio::test]
async fn makes_a_telephone_or_an_echo_mixed_with_the_original() {
    let (_, ran) = run(
        sound_setup("media.voiceEffect", json!({})),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(
        ran[0][3],
        "highpass=f=300,highpass=f=300,lowpass=f=3400,lowpass=f=3400,acrusher=bits=8:mode=log:aa=1:mix=0.3,compand=attacks=0.01:decays=0.2:points=-80/-80|-30/-15|0/-3"
    );
    let (_, ran) = run(
        sound_setup("media.voiceEffect", json!({ "effect": "echo", "mix": 0.3 })),
        sound_probe(),
        voice(),
    )
    .await;
    assert_eq!(
        ran[0][3],
        "[in]asplit[dry][wet];[wet]aecho=in_gain=1:out_gain=1:delays=300:decays=0.4[effect];[dry][effect]amix=inputs=2:weights='0.7 0.3':normalize=0:duration=first[out]"
    );
}

#[tokio::test]
async fn writes_the_room_beside_the_result_and_convolves_with_it() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".opsis/cache/h")).unwrap();
    let setup = Setup {
        kind: "media.voiceEffect",
        params: json!({ "effect": "hall", "mix": 0.25 }),
        project_dir: dir.path(),
        names: &[],
    };
    let (result, ran) = run(setup, sound_probe(), voice()).await;
    result.unwrap();
    let room = dir.path().join(".opsis/cache/h/room.wav");
    let spec = hound::WavReader::open(&room).unwrap().spec();
    assert_eq!((spec.channels, spec.sample_rate), (2, 48000));
    assert_eq!(
        ran[0][3],
        format!(
            "amovie=filename={}[room];[in]asplit[dry][wet];[wet][room]afir=gtype=none:irnorm=-1[effect];[dry][effect]amix=inputs=2:weights='0.75 0.25':normalize=0:duration=first[out]",
            room.to_string_lossy()
        )
    );
}

fn film_clip(frames: u32) -> Value {
    json!({ "type": "clip", "durationInFrames": frames, "tracks": [], "audio": [] })
}

#[tokio::test]
async fn fits_a_song_to_the_films_length_by_stretching_or_looping_it() {
    let inputs = |frames: u32| json!({ "clip": film_clip(frames), "media": { "type": "media", "kind": "audio", "src": "project:assets/song.m4a", "durationMs": 4000 } });
    let (result, ran) = run(
        sound_setup("media.fitMusic", json!({})),
        sound_probe(),
        inputs(126),
    )
    .await;
    assert_eq!(
        ran[0][..6],
        [
            "-i",
            "/p/assets/song.m4a",
            "-filter_complex",
            "[0:a]apad=pad_dur=1,rubberband=tempo=0.952381:pitchq=quality:transients=mixed,atrim=end=4.200,afade=t=out:st=2.200:d=2.000[out]",
            "-map",
            "[out]"
        ]
    );
    assert!(ran[0].last().unwrap().ends_with("fitted.m4a"));
    assert_eq!(
        result.unwrap().measured,
        [Measured::Fitted {
            tempo: Some(4000.0 / 4200.0),
            loops: 0
        }]
    );
    let (result, ran) = run(
        sound_setup(
            "media.fitMusic",
            json!({ "crossfadeMs": 500, "fadeOutMs": 0 }),
        ),
        sound_probe(),
        inputs(300),
    )
    .await;
    assert_eq!(
        ran[0][..7],
        [
            "-i",
            "/p/assets/song.m4a",
            "-i",
            "/p/assets/song.m4a",
            "-i",
            "/p/assets/song.m4a",
            "-filter_complex"
        ]
    );
    assert_eq!(
        ran[0][7],
        "[0:a]atrim=end=4.000,asetpts=PTS-STARTPTS[p0];[1:a]atrim=start=0.000:end=4.000,asetpts=PTS-STARTPTS[p1];[2:a]atrim=start=0.000:end=4.000,asetpts=PTS-STARTPTS[p2];[p0][p1]acrossfade=d=0.500:c1=qsin:c2=qsin[x1];[x1][p2]acrossfade=d=0.500:c1=qsin:c2=qsin[x2];[x2]atrim=end=10.000[out]"
    );
    assert_eq!(
        result.unwrap().measured,
        [Measured::Fitted {
            tempo: None,
            loops: 2
        }]
    );
    let (result, _) = run(
        sound_setup("media.fitMusic", json!({})),
        sound_probe(),
        json!({ "media": { "type": "media", "kind": "audio", "src": "project:assets/song.m4a" } }),
    )
    .await;
    assert_eq!(
        message(result),
        "Connect the film, so the song knows how long to be."
    );
}

#[tokio::test]
async fn generates_each_variation_at_its_own_length() {
    let (out, samples, rate) = made_sound("audio.generate", json!({})).await;
    assert_eq!((rate, samples.len()), (48000, 2 * 38_400));
    assert_eq!(
        serde_json::to_value(&out.outputs["audio"]).unwrap()["durationMs"],
        800
    );
    let (_, longer, _) = made_sound(
        "audio.generate",
        json!({ "preset": "swell", "variation": "longer" }),
    )
    .await;
    assert_eq!(longer.len(), 2 * 288_000);
}

fn sequence_with_two_fades() -> Value {
    let node = |id: &str, kind: &str, params: Value| json!({ "id": id, "type": kind, "position": { "x": 0, "y": 0 }, "params": params });
    let edge = |source: &str, target: &str, port: &str| json!({ "id": format!("{source}-{port}"), "source": source, "sourceHandle": "clip", "target": target, "targetHandle": port });
    let wf: Workflow = serde_json::from_value(json!({
        "version": 1, "id": "x", "name": "x",
        "nodes": [
            node("a", "fx.title", json!({ "clipDuration": 60 })),
            node("b", "fx.title", json!({ "clipDuration": 60 })),
            node("c", "fx.title", json!({ "clipDuration": 60 })),
            node("seq", "timeline.sequence", json!({
                "transition": "fade", "transitionDuration": 15,
                "cuts": [{ "before": "in3", "transition": "fade", "transitionDuration": 30 }]
            }))
        ],
        "edges": [edge("a", "seq", "in1"), edge("b", "seq", "in2"), edge("c", "seq", "in3")]
    }))
    .unwrap();
    let compiled = opsis_compile::compile::compile_workflow(&wf, None);
    json!({ "clip": compiled.values["seq"]["clip"] })
}

#[tokio::test]
async fn makes_one_sound_for_each_transition_length() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".opsis/cache/h")).unwrap();
    let setup = |params: Value| Setup {
        kind: "audio.transitionSounds",
        params,
        project_dir: dir.path(),
        names: &[],
    };
    let (result, ran) = run(setup(json!({})), sound_probe(), sequence_with_two_fades()).await;
    assert_eq!(ran, [] as [Vec<String>; 0]);
    let mut files: Vec<String> = result
        .unwrap()
        .artifacts
        .iter()
        .map(|a| a.path.rsplit('/').next().unwrap().to_owned())
        .collect();
    files.sort();
    assert_eq!(files, ["transition-1000.wav", "transition-500.wav"]);
    let reader =
        hound::WavReader::open(dir.path().join(".opsis/cache/h/transition-500.wav")).unwrap();
    assert_eq!(reader.duration(), 24_000);
    let (result, _) = run(
        setup(json!({ "soundFrom": "file" })),
        sound_probe(),
        sequence_with_two_fades(),
    )
    .await;
    assert_eq!(result.unwrap().artifacts, []);
}

#[tokio::test]
async fn makes_the_riser_at_its_length_and_the_end_hit_only_when_asked() {
    let (out, samples, _) = made_sound("audio.riser", json!({ "lengthMs": 2000 })).await;
    assert_eq!((out.artifacts.len(), samples.len()), (1, 2 * 96_000));
    let (out, _, _) = made_sound("audio.riser", json!({ "endHit": "ding" })).await;
    let files: Vec<&str> = out
        .artifacts
        .iter()
        .map(|a| a.path.rsplit('/').next().unwrap())
        .collect();
    assert_eq!(files, ["riser.wav", "end-hit.wav"]);
}
