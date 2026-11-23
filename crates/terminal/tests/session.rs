use opsis_terminal::cast::EventData;
use opsis_terminal::take::{Take, TakeOptions, record_take};
use opsis_terminal::timeline::{END, START};
use opsis_types::actions::ActionKind;
use opsis_types::terminal::{
    CellTarget, LookIn, PointerButton, ShellChoice, TerminalAction, TerminalSessionValue,
    TerminalStep, WaitFor,
};

const ASK_POSITION: &str = if cfg!(windows) {
    r#"$e = [char]27; [Console]::Write("$e[6n"); $r = ''; do { $c = [Console]::ReadKey($true).KeyChar; $r += $c } until ($c -eq 'R'); 'at:' + $r.Substring($r.IndexOf('[') + 1).TrimEnd('R')"#
} else {
    r#"printf '\033[6n'; IFS= read -rsd R pos; echo "at:${pos#*[}""#
};

const READ_CLICK: &str = if cfg!(windows) {
    r#"Add-Type -Namespace Opsis -Name Console -MemberDefinition '[DllImport("kernel32.dll")] public static extern System.IntPtr GetStdHandle(int h); [DllImport("kernel32.dll")] public static extern bool GetConsoleMode(System.IntPtr h, out int m); [DllImport("kernel32.dll")] public static extern bool SetConsoleMode(System.IntPtr h, int m);'; $h = [Opsis.Console]::GetStdHandle(-10); $m = 0; [void][Opsis.Console]::GetConsoleMode($h, [ref]$m); [void][Opsis.Console]::SetConsoleMode($h, ($m -bor 0x290) -band -bnot 0x40); $e = [char]27; [Console]::Write("$e[?1000h$e[?1006h"); $r = ''; do { $c = [char][Console]::In.Read(); $r += $c } until ($c -eq 'M'); [Console]::Write("$e[?1000l"); 'got:' + $r.Substring(1)"#
} else {
    r"printf '\033[?1000h\033[?1006h'; IFS= read -rsd M m; printf '\033[?1000l'; echo got:${m#?}"
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
        live: None,
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
    let slept = text
        .find("slept-well\r\n")
        .unwrap_or_else(|| panic!("{text:?}"));
    let after = text
        .find("echo after")
        .unwrap_or_else(|| panic!("{text:?}"));
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
    let made = take(vec![step("ask", run(ASK_POSITION))]);
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
                text: READ_CLICK.to_owned() + "\n",
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

#[test]
#[ignore = "needs a shell"]
fn a_tape_plays_and_its_waits_wait_for_the_prompt() {
    let tape = opsis_terminal::tape::parse(
        "Set TypingSpeed 5ms\nType \"sleep 1; echo tape-done\"\nEnter\nWait\nType \"echo after\"\nEnter\nWait\n",
    )
    .unwrap();
    let mapped = opsis_terminal::tape::tape_steps(&tape, "play");
    let started = std::time::Instant::now();
    let made = take(mapped.session.steps);
    assert!(started.elapsed() >= std::time::Duration::from_secs(1));
    let out = output(&made);
    let done = out.find("tape-done").expect("the first command finished");
    let after = out.rfind("after").expect("the second command ran");
    assert!(after > done, "{out}");
}
