use std::path::Path;

use jiff::tz::TimeZone;
use jiff::{Timestamp, ToSpan};
use opsis_qt::{
    AgentAskView, AgentEntryView, AgentOptionView, AgentPanelView, AgentThreadView, AgentToolView,
    AssistantSessionView, Choice, OwnAgentView,
};
use opsis_session::Session;
use opsis_session::agents::AgentChoice;
use opsis_session::assistant::{self, Ask, Entry, PanelState, Status};
use opsis_session::state::AppState;
use opsis_session::tool_calls::{self, Category, Stage};
use opsis_store::assistant::AssistantThread;
use opsis_types::settings::OwnAgent;
use tokio::runtime::Runtime;

use crate::markdown;

const SESSIONS_SHOWN: usize = 20;
const GROUP_PREFIX: &str = "group:";

fn label(choice: &AgentChoice) -> String {
    choice.version.as_ref().map_or_else(
        || choice.name.clone(),
        |version| format!("{} ({version})", choice.name),
    )
}

fn choices(agents: &[AgentChoice]) -> Vec<Choice> {
    agents
        .iter()
        .map(|agent| Choice {
            value: agent.id.clone(),
            label: label(agent),
        })
        .collect()
}

const fn icon(category: Category) -> &'static str {
    match category {
        Category::Read => "eye",
        Category::Edit => "pencil",
        Category::Search => "search",
        Category::Command => "square-terminal",
        Category::Script => "code",
        Category::Other => "plug",
    }
}

fn summary(categories: &[Category]) -> String {
    let count = categories.len();
    let plural = |one: &str, many: &str| if count == 1 { one } else { many }.to_owned();
    let same = categories.windows(2).all(|pair| pair[0] == pair[1]);
    match categories.first().copied().filter(|_| same) {
        Some(Category::Read) => format!("Read {count} {}", plural("file", "files")),
        Some(Category::Edit) => format!("Made {count} {}", plural("change", "changes")),
        Some(Category::Search) => format!("Searched {count} {}", plural("time", "times")),
        Some(Category::Command) => format!("Ran {count} {}", plural("command", "commands")),
        Some(Category::Script) => format!("Ran {count} {}", plural("script", "scripts")),
        _ => format!("Used {count} {}", plural("tool", "tools")),
    }
}

#[derive(Debug, Clone, Copy)]
struct Shown<'a> {
    app: &'a Session,
    project: Option<&'a Path>,
}

struct Tool<'a> {
    id: &'a str,
    category: Category,
    view: AgentToolView,
}

fn stage(status: &str) -> Stage {
    match status {
        "failed" => Stage::Failed,
        "done" => Stage::Done,
        _ => Stage::Running,
    }
}

fn tool<'a>(shown: Shown<'_>, entry: &'a Entry) -> Option<Tool<'a>> {
    let Entry::Tool {
        id,
        kind,
        title,
        input,
        status,
        output,
    } = entry
    else {
        return None;
    };
    let said = tool_calls::sentence(kind, title, input.as_ref(), stage(status), shown.project);
    Some(Tool {
        id,
        category: said.category,
        view: AgentToolView {
            id: id.clone(),
            label: said.text,
            icon: icon(said.category).to_owned(),
            failed: status == "failed",
            running: status == "running" || status == "pending",
            output: shown.project.map_or_else(
                || output.clone(),
                |project| output.replace(&format!("{}/", project.to_string_lossy()), ""),
            ),
            expanded: assistant::is_expanded(shown.app, id),
        },
    })
}

fn text_entry(kind: &str, text: String) -> AgentEntryView {
    AgentEntryView {
        kind: kind.to_owned(),
        id: String::new(),
        text,
        icon: String::new(),
        failed: false,
        running: false,
        expanded: false,
        tools: Vec::new(),
    }
}

fn group(shown: Shown<'_>, tools: Vec<Tool<'_>>) -> AgentEntryView {
    let id = format!("{GROUP_PREFIX}{}", tools[0].id);
    let categories: Vec<Category> = tools.iter().map(|tool| tool.category).collect();
    let same = categories.windows(2).all(|pair| pair[0] == pair[1]);
    let views: Vec<AgentToolView> = tools.into_iter().map(|tool| tool.view).collect();
    AgentEntryView {
        kind: "tools".to_owned(),
        text: summary(&categories),
        icon: if same { icon(categories[0]) } else { "plug" }.to_owned(),
        failed: views.iter().any(|tool| tool.failed),
        running: views.iter().any(|tool| tool.running),
        expanded: assistant::is_expanded(shown.app, &id),
        id,
        tools: views,
    }
}

fn entries(shown: Shown<'_>, entries: &[Entry], asked: &[Ask]) -> Vec<AgentEntryView> {
    let mut made = Vec::new();
    let mut tools = Vec::new();
    for entry in entries {
        if let Some(found) = tool(shown, entry) {
            if !asked.iter().any(|ask| ask.id == found.id) {
                tools.push(found);
            }
            continue;
        }
        if !tools.is_empty() {
            made.push(group(shown, std::mem::take(&mut tools)));
        }
        made.push(match entry {
            Entry::User(text) => text_entry("user", text.clone()),
            Entry::Agent(text) => text_entry("agent", markdown::html(text)),
            Entry::Notice(text) | Entry::Tool { title: text, .. } => {
                text_entry("notice", text.clone())
            }
        });
    }
    if !tools.is_empty() {
        made.push(group(shown, tools));
    }
    made
}

fn asked_tool<'a>(entries: &'a [Entry], id: &str) -> Option<(&'a str, &'a str)> {
    entries.iter().find_map(|entry| match entry {
        Entry::Tool {
            id: tool,
            kind,
            title,
            ..
        } if tool == id => Some((kind.as_str(), title.as_str())),
        _ => None,
    })
}

fn ask(shown: Shown<'_>, ask: &Ask, entries: &[Entry]) -> AgentAskView {
    let kind = asked_tool(entries, &ask.id).map_or("other", |(kind, _)| kind);
    let said = tool_calls::sentence(
        kind,
        &ask.title,
        ask.input.as_ref(),
        Stage::Asking,
        shown.project,
    );
    AgentAskView {
        id: ask.id.clone(),
        title: if ask.sign_in {
            ask.title.clone()
        } else {
            said.text
        },
        detail: tool_calls::script(ask.input.as_ref()),
        icon: if ask.sign_in {
            "key-round"
        } else {
            icon(said.category)
        }
        .to_owned(),
        sign_in: ask.sign_in,
        options: ask
            .options
            .iter()
            .map(|option| AgentOptionView {
                id: option.id.clone(),
                name: option.name.clone(),
                kind: option.kind.clone(),
            })
            .collect(),
    }
}

fn now() -> Timestamp {
    Timestamp::now()
}

fn ago(at: Timestamp, now: Timestamp) -> String {
    let minutes = (now.as_millisecond() - at.as_millisecond()).max(0) / 60_000;
    match minutes {
        0 => "now".to_owned(),
        1..60 => format!("{minutes}m"),
        60..1440 => format!("{}h", minutes / 60),
        1440..10_080 => format!("{}d", minutes / 1440),
        _ => format!("{}w", minutes / 10_080),
    }
}

fn day_group(at: Timestamp, now: Timestamp, zone: &TimeZone) -> &'static str {
    let day = at.to_zoned(zone.clone()).date();
    let today = now.to_zoned(zone.clone()).date();
    if day >= today {
        "Today"
    } else if today
        .checked_sub(1.day())
        .is_ok_and(|yesterday| day >= yesterday)
    {
        "Yesterday"
    } else {
        "Earlier"
    }
}

fn thread_state(state: &PanelState) -> &'static str {
    if state.asks.iter().any(|ask| !ask.sign_in) {
        "ask"
    } else if state.status == Status::Working {
        "work"
    } else {
        ""
    }
}

fn threads(state: &PanelState, past: Vec<AssistantThread>) -> Vec<AgentThreadView> {
    let now = now();
    let zone = TimeZone::system();
    let live = state.session.clone().map(|id| AgentThreadView {
        title: if state.title.is_empty() {
            assistant::NEW_THREAD.to_owned()
        } else {
            state.title.clone()
        },
        when: "now".to_owned(),
        group: "Today".to_owned(),
        state: thread_state(state).to_owned(),
        id,
    });
    let earlier = past
        .into_iter()
        .filter(|thread| Some(&thread.id) != state.session.as_ref())
        .map(|thread| {
            let at = Timestamp::from_millisecond(thread.updated_at).unwrap_or(now);
            AgentThreadView {
                id: thread.id,
                title: thread.title.lines().next().unwrap_or_default().to_owned(),
                when: ago(at, now),
                group: day_group(at, now, &zone).to_owned(),
                state: String::new(),
            }
        });
    live.into_iter().chain(earlier).collect()
}

const fn status(status: &Status) -> &'static str {
    match status {
        Status::Off => "",
        Status::Starting => "starting",
        Status::SignIn => "sign-in",
        Status::Ready => "ready",
        Status::Working => "working",
        Status::Ended(_) => "ended",
    }
}

pub fn view(app: &Session, runtime: &Runtime) -> AgentPanelView {
    let state = assistant::panel_state(app);
    let open = assistant::open_thread_id(app).unwrap_or_default();
    let live = !open.is_empty() && state.session.as_deref() == Some(open.as_str());
    let listed = runtime.block_on(assistant::threads(app));
    let shown_threads = threads(&state, listed);
    let past = if open.is_empty() || live {
        Vec::new()
    } else {
        runtime.block_on(assistant::past_entries(app, &open))
    };
    let title = shown_threads
        .iter()
        .find(|thread| thread.id == open)
        .map_or_else(String::new, |thread| thread.title.clone());
    let project = app.state::<AppState>().editor().dir();
    let shown = Shown {
        app,
        project: project.as_deref(),
    };
    AgentPanelView {
        assistant_on: app.state::<AppState>().settings().assistant.enabled,
        has_project: app.state::<AppState>().editor().dir().is_some(),
        thread: open,
        live,
        title,
        threads: shown_threads,
        status: if live { status(&state.status) } else { "" }.to_owned(),
        ended: match &state.status {
            Status::Ended(message) if live => message.clone(),
            _ => String::new(),
        },
        working_since: state.working_since.filter(|_| live).unwrap_or_default(),
        modes: if live {
            state
                .modes
                .iter()
                .map(|mode| Choice {
                    value: mode.id.clone(),
                    label: mode.name.clone(),
                })
                .collect()
        } else {
            Vec::new()
        },
        mode: state.mode.clone().unwrap_or_default(),
        entries: if live {
            entries(shown, &state.entries, &state.asks)
        } else {
            entries(shown, &past, &[])
        },
        asks: if live {
            state
                .asks
                .iter()
                .map(|asked| ask(shown, asked, &state.entries))
                .collect()
        } else {
            Vec::new()
        },
    }
}

pub fn found_agents(app: &Session) -> Vec<Choice> {
    choices(&assistant::agents(app))
}

pub fn own_agents(own: &[OwnAgent]) -> Vec<OwnAgentView> {
    own.iter()
        .map(|agent| OwnAgentView {
            name: agent.name.clone(),
            command: agent.command.clone(),
            args: agent.args.clone(),
        })
        .collect()
}

pub fn sessions(app: &Session, runtime: &Runtime) -> Vec<AssistantSessionView> {
    let db = app.state::<AppState>().db();
    let now = now();
    let listed = runtime.block_on(async {
        let sessions = db.assistant_sessions(None).await?;
        let mut shown = Vec::new();
        for session in sessions.into_iter().take(SESSIONS_SHOWN) {
            let messages = db.assistant_messages(&session.id).await?.len();
            let at = Timestamp::from_millisecond(session.updated_at).unwrap_or(now);
            shown.push(AssistantSessionView {
                agent: session.agent,
                project: session.project,
                updated: format!("{} ago", ago(at, now)),
                messages: u32::try_from(messages).unwrap_or(u32::MAX),
            });
        }
        Ok::<_, opsis_store::db::StoreError>(shown)
    });
    listed.unwrap_or_else(|error| {
        log::warn!("listing the assistant's sessions: {error}");
        Vec::new()
    })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const CATEGORIES: [Category; 6] = [
        Category::Read,
        Category::Edit,
        Category::Search,
        Category::Command,
        Category::Script,
        Category::Other,
    ];

    proptest! {
        #[test]
        fn a_time_ago_is_one_number_and_one_unit(ago_ms in 0_i64..400 * 86_400_000) {
            let now = Timestamp::now();
            let at = Timestamp::from_millisecond(now.as_millisecond() - ago_ms).unwrap();
            let said = ago(at, now);
            prop_assert!(
                said == "now" || (said[..said.len() - 1].parse::<i64>().is_ok() && said.ends_with(['m', 'h', 'd', 'w'])),
                "{said}"
            );
        }

        #[test]
        fn a_tool_summary_counts_every_tool(picks in proptest::collection::vec(0_usize..CATEGORIES.len(), 1..12)) {
            let categories: Vec<Category> = picks.iter().map(|pick| CATEGORIES[*pick]).collect();
            let said = summary(&categories);
            let count = categories.len().to_string();
            prop_assert!(said.split(' ').nth(1) == Some(count.as_str()), "{said}");
        }

        #[test]
        fn a_thread_day_never_comes_after_today(ago_ms in 0_i64..30 * 86_400_000) {
            let now = Timestamp::now();
            let at = Timestamp::from_millisecond(now.as_millisecond() - ago_ms).unwrap();
            let group = day_group(at, now, &TimeZone::UTC);
            prop_assert!(["Today", "Yesterday", "Earlier"].contains(&group));
            if ago_ms > 2 * 86_400_000 {
                prop_assert_eq!(group, "Earlier");
            }
        }
    }
}
