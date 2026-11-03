use opsis_terminal::cast::EventData;
use opsis_terminal::take::{Take, TakeOptions, record_take};
use opsis_terminal::timeline::{END, START};
use opsis_types::actions::ActionKind;
use opsis_types::terminal::{
    CellTarget, LookIn, PointerButton, ShellChoice, TerminalAction, TerminalSessionValue,
    TerminalStep, WaitFor,
};

fn step(id: &str, action: TerminalAction) -> TerminalStep {
    TerminalStep {
        id: id.to_owned(),
        node_id: id.to_owned(),
        label: id.to_owned(),
        action,
        wait_after_ms: 100,
    }
}

fn run(command: &str) -> TerminalAction {
    TerminalAction::Run {
        command: command.to_owned(),
        wait: WaitFor::Prompt,
        limit_ms: 10_000,
        zoom_output: true,
    }
}

fn take(steps: Vec<TerminalStep>) -> Take {
    let dir = tempfile::tempdir().unwrap();
    let value = TerminalSessionValue {
        shell: ShellChoice::Clean,
        folder: String::new(),
        cols: 80,
        rows: 24,
        env: Vec::new(),
        typing_ms: 5,
        pointer_ms: 100,
        steps,
    };
    let mut log = |line: String| eprintln!("{line}");
    record_take(&mut TakeOptions {
        session: &value,
        project_dir: dir.path(),
        temp: dir.path(),
        name: "Test",
        pause_before_ms: 0,
        keep_after_ms: 200,
        cancelled: &|| false,
        log: &mut log,
    })
    .unwrap()
}

fn output(take: &Take) -> String {
    take.events
        .iter()
        .filter_map(|e| match &e.data {
            EventData::Output(text) => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

fn markers(take: &Take) -> Vec<String> {
    take.events
        .iter()
        .filter_map(|e| match &e.data {
            EventData::Marker(label) => Some(label.clone()),
            _ => None,
        })
        .collect()
}

#[test]
#[ignore = "needs a shell"]
fn waits_for_each_command_to_finish_through_the_prompt_marks() {
    let made = take(vec![
        step("start", TerminalAction::StartRecording),
        step("slow", run("sleep 1; echo slept-well")),
        step("after", run("echo after")),
    ]);
    let text = output(&made);
    assert!(text.contains("\x1b]133;A") && text.contains("\x1b]133;D;0"));
    let slept = text.find("slept-well\r\n").unwrap();
    let after = text.find("echo after").unwrap();
    assert!(slept < after, "{text}");
    let marks = markers(&made);
    assert_eq!(marks.first().map(String::as_str), Some("opsis:step:start"));
    assert!(marks.contains(&START.to_owned()) && marks.contains(&END.to_owned()));
    let typed: Vec<_> = made
        .log
        .actions
        .iter()
        .filter(|a| a.kind == ActionKind::Type)
        .collect();
    assert_eq!(typed.len(), 2);
    assert_eq!(
        typed[0].key_times.as_ref().unwrap().len(),
        "sleep 1; echo slept-well".len()
    );
    assert!(made.log.duration_ms > 1000.0);
    assert_ne!(
        made.log.caret_track,
        [] as [opsis_types::actions::CaretSample; 0]
    );
}

#[test]
#[ignore = "needs a shell"]
fn answers_the_programs_questions_about_the_terminal() {
    let made = take(vec![step(
        "ask",
        run(r#"printf '\033[6n'; IFS= read -rsd R pos; echo "at:${pos#*[}""#),
    )]);
    assert!(output(&made).contains("at:"), "{}", output(&made));
    let at = output(&made);
    let answer = at.rsplit("at:").next().unwrap();
    assert!(answer.starts_with(|c: char| c.is_ascii_digit()), "{answer}");
}

#[test]
#[ignore = "needs a shell"]
fn clicks_reach_a_program_that_reads_the_mouse() {
    let made = take(vec![
        step(
            "listen",
            TerminalAction::Type {
                text: r"printf '\033[?1000h\033[?1006h'; IFS= read -rsd M m; printf '\033[?1000l'; echo got:${m#?}".to_owned()
                    + "\n",
                speed_ms: Some(2),
            },
        ),
        step(
            "click",
            TerminalAction::Click {
                target: CellTarget::Cell { col: 4, row: 10 },
                button: PointerButton::Left,
                double: false,
            },
        ),
        step(
            "wait",
            TerminalAction::WaitText {
                pattern: "got:".to_owned(),
                scope: LookIn::Screen,
                limit_ms: 5000,
            },
        ),
    ]);
    assert!(output(&made).contains("got:[<0;5;11"), "{}", output(&made));
    let click = made
        .log
        .actions
        .iter()
        .find(|a| a.kind == ActionKind::Click)
        .unwrap();
    assert_eq!(click.point.map(|p| (p.x, p.y)), Some((4.5, 10.5)));
    assert!(
        made.log
            .cursor_track
            .as_ref()
            .is_some_and(|t| !t.is_empty())
    );
}

#[test]
#[ignore = "needs a shell"]
fn keeps_hidden_steps_off_the_visible_timeline() {
    let made = take(vec![
        step("hide", TerminalAction::Hide),
        step("setup", run("cd / && clear")),
        step("show", TerminalAction::Show),
        step("start", TerminalAction::StartRecording),
        step("seen", run("echo shown")),
    ]);
    let kinds: Vec<&str> = made
        .log
        .actions
        .iter()
        .filter_map(|a| a.node_id.as_deref())
        .collect();
    assert!(!kinds.contains(&"setup"), "{kinds:?}");
    assert!(kinds.contains(&"seen"));
}
