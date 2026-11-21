use opsis_store::assistant::{
    AssistantEvent, AssistantMessage, AssistantRequest, AssistantSession, AssistantTurn, Recorded,
};
use opsis_store::db::Database;
use proptest::prelude::*;

type Projected = (
    Vec<AssistantSession>,
    Vec<(
        Vec<AssistantMessage>,
        Vec<AssistantTurn>,
        Vec<AssistantRequest>,
    )>,
);

async fn projected(db: &Database) -> Projected {
    let sessions = db.assistant_sessions(None).await.unwrap();
    let mut each = Vec::new();
    for id in ["s0", "s1"] {
        each.push((
            db.assistant_messages(id).await.unwrap(),
            db.assistant_turns(id).await.unwrap(),
            db.assistant_requests(id).await.unwrap(),
        ));
    }
    (sessions, each)
}

fn event() -> impl Strategy<Value = AssistantEvent> {
    let text = "[a-z ]{0,8}";
    let request = "r[0-3]";
    prop_oneof![
        proptest::option::of("[a-f0-9]{4}")
            .prop_map(|agent_session| AssistantEvent::SessionStarted { agent_session }),
        text.prop_map(|text| AssistantEvent::UserMessage { text }),
        text.prop_map(|text| AssistantEvent::AgentText { text }),
        Just(AssistantEvent::TurnStarted),
        "(endTurn|cancelled|refusal)".prop_map(|stop| AssistantEvent::TurnEnded { stop }),
        (request, "(permission|signIn)", text).prop_map(|(request, kind, title)| {
            AssistantEvent::RequestAsked {
                request,
                kind,
                title,
            }
        }),
        (request, "(allow|reject)")
            .prop_map(|(request, answer)| AssistantEvent::RequestAnswered { request, answer }),
    ]
}

fn recorded() -> impl Strategy<Value = Recorded> {
    (0..2_usize, 0..1000_i64, event()).prop_map(|(session, at, event)| Recorded {
        session: format!("s{session}"),
        project: "/p".to_owned(),
        agent: "opsis-agent".to_owned(),
        at,
        event,
    })
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn replaying_the_events_builds_the_same_projections_after_a_restart(events in proptest::collection::vec(recorded(), 0..40)) {
        runtime().block_on(async {
            let dir = tempfile::tempdir().unwrap();
            let file = dir.path().join("opsis.db");
            let db = Database::open(&file).await.unwrap();
            for recorded in &events {
                db.record_assistant_event(recorded).await.unwrap();
            }
            let live = projected(&db).await;
            drop(db);
            let reopened = Database::open(&file).await.unwrap();
            assert_eq!(projected(&reopened).await, live);
            reopened.rebuild_assistant_projections().await.unwrap();
            assert_eq!(projected(&reopened).await, live);
        });
    }
}

#[tokio::test]
async fn agent_text_streamed_in_pieces_is_one_message_until_the_turn_ends() {
    let db = Database::in_memory().await.unwrap();
    let at = |event| Recorded {
        session: "s0".into(),
        project: "/p".into(),
        agent: "opsis-agent".into(),
        at: 1,
        event,
    };
    for event in [
        AssistantEvent::SessionStarted {
            agent_session: Some("a1".into()),
        },
        AssistantEvent::UserMessage {
            text: "Rename it".into(),
        },
        AssistantEvent::TurnStarted,
        AssistantEvent::AgentText { text: "Re".into() },
        AssistantEvent::AgentText {
            text: "named.".into(),
        },
        AssistantEvent::TurnEnded {
            stop: "endTurn".into(),
        },
        AssistantEvent::AgentText {
            text: "Later.".into(),
        },
    ] {
        db.record_assistant_event(&at(event)).await.unwrap();
    }
    let texts: Vec<(String, String)> = db
        .assistant_messages("s0")
        .await
        .unwrap()
        .into_iter()
        .map(|m| (m.role, m.text))
        .collect();
    assert_eq!(
        texts,
        [
            ("user".to_owned(), "Rename it".to_owned()),
            ("agent".to_owned(), "Renamed.".to_owned()),
            ("agent".to_owned(), "Later.".to_owned()),
        ]
    );
    db.clear_assistant_log().await.unwrap();
    assert_eq!(db.assistant_sessions(None).await.unwrap(), Vec::new());
}
