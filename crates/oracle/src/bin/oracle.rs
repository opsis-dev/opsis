use std::path::{Path, PathBuf};

use opsis_compile::asset_ref::FILM_FOLDER;
use opsis_compile::elements::{ElementMeta, elements};
use opsis_oracle::boxes::BoxReader;
use opsis_oracle::cases::{CASE_HEIGHT, CASE_WIDTH, case_ids, case_spec, case_url, cases_dir};
use opsis_render::element_spec::{element_spec, frames_at};

const ORACLE_POSITIONS: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];
use opsis_oracle::films::{EXPORT_FOLDER, ExampleRun, example_names, prepare_example, run_example};
use opsis_oracle::host::{film_page, headless_shell, oracle_page, serve};
use opsis_oracle::pictures::{Drawing, ORACLE_GL, draw_frames};
use serde_json::{Value, json};

const COMMIT_ENV: &str = "OPSIS_ORACLE_COMMIT";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    Elements,
    Cases,
    Boxes,
    Films,
}

impl Part {
    fn parse(word: &str) -> Vec<Self> {
        match word {
            "elements" => vec![Self::Elements],
            "cases" => vec![Self::Cases],
            "boxes" => vec![Self::Boxes],
            "films" => vec![Self::Films],
            "all" => vec![Self::Cases, Self::Boxes, Self::Elements, Self::Films],
            other => panic!("Unknown part “{other}”: say elements, cases, boxes, films or all."),
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Elements => "elements",
            Self::Cases => "cases",
            Self::Boxes => "boxes",
            Self::Films => "films",
        }
    }
}

struct Oracle {
    out: PathBuf,
    shell: PathBuf,
    temp: PathBuf,
    base: String,
    ffmpeg: PathBuf,
    only: String,
    failures: Vec<String>,
}

fn write_json(path: &Path, value: &Value) {
    let text = serde_json::to_string_pretty(value).expect("JSON text");
    std::fs::write(path, text).unwrap_or_else(|e| panic!("writing {}: {e}", path.display()));
}

fn frame_file(dir: &Path, frame: u32) -> PathBuf {
    dir.join(format!("{frame:05}.png"))
}

impl Oracle {
    fn fail(&mut self, what: &str, error: &dyn std::fmt::Display) {
        println!("FAIL {what}");
        self.failures.push(format!("{what}: {error}"));
    }

    async fn element(&mut self, meta: &ElementMeta) -> Option<Value> {
        let spec = match element_spec(meta, &self.base) {
            Ok(spec) => spec,
            Err(error) => {
                self.fail(&meta.key, &error);
                return None;
            }
        };
        let dir = self.out.join("elements").join(&meta.key);
        std::fs::create_dir_all(&dir).expect("an element folder");
        write_json(
            &dir.join("spec.json"),
            &serde_json::to_value(&spec).expect("a spec"),
        );
        let frames = frames_at(spec.duration_in_frames, &ORACLE_POSITIONS);
        let files: Vec<(u32, PathBuf)> = frames.iter().map(|&f| (f, frame_file(&dir, f))).collect();
        let url = format!("{}{FILM_FOLDER}/index.html", self.base);
        let drawing = Drawing {
            shell: &self.shell,
            temp: &self.temp,
            url: &url,
            spec: &spec,
        };
        if files.iter().any(|(_, p)| !p.exists())
            && let Err(error) = draw_frames(&drawing, &files).await
        {
            self.fail(&meta.key, &error);
            return None;
        }
        println!("ok   {}", meta.key);
        Some(
            json!({ "key": meta.key, "durationInFrames": spec.duration_in_frames, "frames": frames }),
        )
    }

    async fn elements(&mut self) {
        let chosen: Vec<&ElementMeta> = elements()
            .iter()
            .filter(|m| m.key.contains(&self.only))
            .collect();
        let mut drawn = Vec::new();
        for meta in chosen {
            drawn.extend(self.element(meta).await);
        }
        let file = self.out.join("elements.json");
        let all = merged(&file, "elements", "key", drawn);
        write_json(
            &file,
            &json!({ "positions": ORACLE_POSITIONS, "elements": all }),
        );
    }

    async fn cases(&mut self) {
        let spec = case_spec().expect("the case page's fonts");
        let ids: Vec<String> = case_ids(&cases_dir())
            .expect("the case list")
            .into_iter()
            .filter(|id| id.contains(&self.only))
            .collect();
        let mut drawn = Vec::new();
        for id in &ids {
            let file = self.out.join("cases").join(format!("{id}.png"));
            std::fs::create_dir_all(file.parent().expect("a case group")).expect("a case folder");
            let url = case_url(&self.base, id);
            let drawing = Drawing {
                shell: &self.shell,
                temp: &self.temp,
                url: &url,
                spec: &spec,
            };
            match draw_frames(&drawing, &[(0, file)]).await {
                Ok(()) => {
                    println!("ok   {id}");
                    drawn.push(json!(id));
                }
                Err(error) => self.fail(id, &error),
            }
        }
        let file = self.out.join("cases.json");
        let all = merged(&file, "cases", "id", drawn);
        write_json(
            &file,
            &json!({ "width": CASE_WIDTH, "height": CASE_HEIGHT, "fonts": spec.fonts, "cases": all }),
        );
    }

    async fn boxes(&mut self) {
        let spec = case_spec().expect("the case page's fonts");
        let ids: Vec<String> = case_ids(&cases_dir())
            .expect("the case list")
            .into_iter()
            .filter(|id| id.contains(&self.only))
            .collect();
        let reader = match BoxReader::open(&self.shell, &self.temp, CASE_WIDTH, CASE_HEIGHT).await {
            Ok(reader) => reader,
            Err(error) => {
                self.fail("boxes", &error);
                return;
            }
        };
        for id in &ids {
            let file = self.out.join("cases").join(format!("{id}.boxes.json"));
            std::fs::create_dir_all(file.parent().expect("a case group")).expect("a case folder");
            match reader.boxes(&case_url(&self.base, id), &spec).await {
                Ok(boxes) => {
                    write_json(&file, &boxes);
                    println!("ok   {id} boxes");
                }
                Err(error) => self.fail(id, &error),
            }
        }
        reader.close().await;
    }

    async fn film(&mut self, name: &str) -> Option<Value> {
        let exported = self.out.join("films").join(name);
        let project = self.out.join("projects").join(name);
        if !exported.is_dir() {
            if project.exists() {
                std::fs::remove_dir_all(&project).expect("clearing an unfinished project");
            }
            let run = async {
                let workflow = prepare_example(name, &project)?;
                let temp = self.temp.join(name);
                std::fs::create_dir_all(&temp).expect("a run temp folder");
                run_example(
                    &ExampleRun {
                        project: &project,
                        temp: &temp,
                        ffmpeg: &self.ffmpeg,
                    },
                    workflow,
                )
                .await
            };
            if let Err(error) = run.await {
                self.fail(name, &error);
                return None;
            }
            copy_exports(&project.join(EXPORT_FOLDER), &exported);
        }
        let files: Vec<String> = std::fs::read_dir(&exported)
            .expect("the film folder")
            .filter_map(Result::ok)
            .filter_map(|e| e.file_name().to_str().map(str::to_owned))
            .collect();
        println!("ok   {name}");
        Some(json!({ "example": name, "files": files }))
    }

    async fn films(&mut self) {
        let names: Vec<String> = example_names()
            .expect("the examples")
            .into_iter()
            .filter(|n| n.contains(&self.only))
            .collect();
        let mut drawn = Vec::new();
        for name in &names {
            drawn.extend(self.film(name).await);
        }
        let file = self.out.join("films.json");
        let films = merged(&file, "films", "example", drawn);
        write_json(&file, &json!({ "films": films }));
    }
}

fn entry_id(entry: &Value, id: &str) -> String {
    entry
        .get(id)
        .unwrap_or(entry)
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

fn merged(file: &Path, list: &str, id: &str, drawn: Vec<Value>) -> Vec<Value> {
    let earlier: Vec<Value> = std::fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|value| value.get(list).and_then(Value::as_array).cloned())
        .unwrap_or_default();
    let fresh: Vec<String> = drawn.iter().map(|e| entry_id(e, id)).collect();
    let mut all: Vec<Value> = earlier
        .into_iter()
        .filter(|e| !fresh.contains(&entry_id(e, id)))
        .chain(drawn)
        .collect();
    all.sort_by_key(|e| entry_id(e, id));
    all
}

fn copy_exports(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a film folder");
    for entry in std::fs::read_dir(from)
        .expect("the exports")
        .filter_map(Result::ok)
    {
        if entry.path().is_file() {
            std::fs::copy(entry.path(), to.join(entry.file_name())).expect("a copied export");
        }
    }
}

fn record(out: &Path, shell: &Path, parts: &[Part]) {
    let file = out.join("oracle.json");
    let mut recorded: Value = std::fs::read_to_string(&file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({}));
    let commit = std::env::var(COMMIT_ENV).unwrap_or_default();
    recorded["browser"] = json!(shell);
    recorded["drawWith"] = json!(ORACLE_GL);
    recorded["elementPositions"] = json!(ORACLE_POSITIONS);
    for part in parts {
        recorded["parts"][part.name()] = json!(commit);
    }
    write_json(&file, &recorded);
}

fn usage() -> ! {
    eprintln!(
        "usage: oracle <out folder> <FFmpeg program> <elements|cases|boxes|films|all> [part of a name]"
    );
    std::process::exit(2);
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    env_logger::init();
    let mut args = std::env::args().skip(1);
    let (Some(out), Some(ffmpeg), Some(what)) = (args.next(), args.next(), args.next()) else {
        usage();
    };
    let out = PathBuf::from(out);
    let temp = out.join("tmp");
    let project = temp.join("project");
    std::fs::create_dir_all(&project).expect("the oracle folder");
    let server = serve(&project, film_page().merge(oracle_page())).await;
    let shell = headless_shell();
    let parts = Part::parse(&what);
    record(&out, &shell, &parts);
    let mut oracle = Oracle {
        out: out.clone(),
        shell,
        temp: temp.clone(),
        base: server.url.clone(),
        ffmpeg: PathBuf::from(ffmpeg),
        only: args.next().unwrap_or_default(),
        failures: Vec::new(),
    };
    for part in parts {
        match part {
            Part::Elements => oracle.elements().await,
            Part::Cases => oracle.cases().await,
            Part::Boxes => oracle.boxes().await,
            Part::Films => oracle.films().await,
        }
    }
    if let Err(error) = std::fs::remove_dir_all(&temp) {
        eprintln!("couldn't remove {}: {error}", temp.display());
    }
    if oracle.failures.is_empty() {
        println!("\nThe oracle is in {}", out.display());
    } else {
        eprintln!("\n{}", oracle.failures.join("\n"));
        std::process::exit(1);
    }
}
