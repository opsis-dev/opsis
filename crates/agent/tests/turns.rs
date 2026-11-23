mod support;

use std::time::Duration;

use agent_client_protocol::Client;
use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::util::MatchDispatch;
use agent_client_protocol::v1::schema::{
    CancelNotification, ContentBlock, ContentChunk, InitializeRequest, SessionNotification,
    SessionUpdate, StopReason,
};
use agent_client_protocol::v1::{SessionMessage, SessionSetup};
use support::*;

#[tokio::test(flavor = "multi_thread")]
async fn a_turn_asks_before_writing_runs_the_tool_and_answers_then_loads_again() {
    let user_data = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let server = model_server(writes_notes()).await;
    log_in(user_data.path(), &server);
    let folder_path = folder.path().to_path_buf();
    let session_id = Client
        .v1()
        .connect_with(agent(user_data.path()), async |cx| {
            cx.send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await?;
            cx.build_session(&folder_path)
                .block_task()
                .run_until(async |SessionSetup { mut session, .. }| {
                    session.send_prompt("Write hello into notes.txt")?;
                    let turned = read_turn(&mut session).await;
                    assert_eq!(turned.stop, Some(StopReason::EndTurn));
                    assert_eq!(turned.asked, 1, "{turned:?}");
                    assert_eq!(turned.tools, 1);
                    assert_eq!(turned.text, "Done.");
                    Ok(session.session_id().clone())
                })
                .await
        })
        .await
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(folder.path().join("notes.txt")).unwrap(),
        "hello"
    );
    let logged = std::fs::read_to_string(
        user_data
            .path()
            .join("logs")
            .join("assistant")
            .join("opsis-agent.ndjson"),
    )
    .unwrap();
    assert!(logged.contains("\"modelRequest\"") && logged.contains("\"modelReply\""));
    assert!(
        logged
            .lines()
            .all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok())
    );
    let replayed = Client
        .v1()
        .connect_with(agent(user_data.path()), async |cx| {
            cx.send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await?;
            let SessionSetup { mut session, .. } = cx
                .load_session(session_id.clone(), &folder_path)
                .block_task()
                .start_session()
                .await?;
            let mut replayed = String::new();
            while let Ok(Ok(SessionMessage::Message(dispatch))) =
                tokio::time::timeout(Duration::from_secs(1), session.read_update()).await
            {
                MatchDispatch::new(dispatch.into_raw())
                    .if_notification(async |notification: SessionNotification| {
                        if let SessionUpdate::AgentMessageChunk(ContentChunk {
                            content: ContentBlock::Text(text),
                            ..
                        })
                        | SessionUpdate::UserMessageChunk(ContentChunk {
                            content: ContentBlock::Text(text),
                            ..
                        }) = notification.update
                        {
                            replayed.push_str(&text.text);
                            replayed.push('\n');
                        }
                        Ok(())
                    })
                    .await
                    .otherwise_ignore()?;
            }
            Ok(replayed)
        })
        .await
        .unwrap();
    assert_eq!(replayed, "Write hello into notes.txt\nDone.\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn cancelling_ends_the_turn_while_the_model_is_still_answering() {
    let user_data = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let server = model_server(says_done().set_delay(Duration::from_secs(30))).await;
    log_in(user_data.path(), &server);
    let folder_path = folder.path().to_path_buf();
    let stop = Client
        .v1()
        .connect_with(agent(user_data.path()), async |cx| {
            cx.send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await?;
            cx.build_session(&folder_path)
                .block_task()
                .run_until(async |SessionSetup { mut session, .. }| {
                    session.send_prompt("Say something")?;
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    session
                        .connection()
                        .send_notification(CancelNotification::new(session.session_id().clone()))?;
                    let turned =
                        tokio::time::timeout(Duration::from_secs(10), read_turn(&mut session))
                            .await
                            .expect("the turn ended");
                    Ok(turned.stop)
                })
                .await
        })
        .await
        .unwrap();
    assert_eq!(stop, Some(StopReason::Cancelled));
}

#[tokio::test(flavor = "multi_thread")]
async fn allowing_always_stops_asking_for_the_same_tool() {
    let server = scripted(vec![
        calls("write", r#"{"path":"a.txt","content":"one"}"#),
        calls("write", r#"{"path":"b.txt","content":"two"}"#),
    ])
    .await;
    let (turned, _, folder) = one_turn(&server, None, "allow-always", None).await;
    assert_eq!((turned.asked, turned.tools), (1, 2), "{turned:?}");
    assert_eq!(
        std::fs::read_to_string(folder.path().join("b.txt")).unwrap(),
        "two"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_command_that_only_reads_runs_without_asking() {
    let server = scripted(vec![calls("shell", r#"{"command":"ls"}"#)]).await;
    let (turned, _, _) = one_turn(&server, None, "reject", None).await;
    assert_eq!((turned.asked, turned.tools), (0, 1), "{turned:?}");
    assert_eq!(turned.stop, Some(StopReason::EndTurn));
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn full_mode_never_asks_and_a_background_command_reports_when_it_ends() {
    let server = scripted(vec![calls(
        "shell",
        r#"{"command":"sleep 1 && echo from-the-background","background":true}"#,
    )])
    .await;
    let (turned, later, _) =
        one_turn(&server, Some("full"), "reject", Some("from-the-background")).await;
    assert_eq!((turned.asked, turned.tools), (0, 1), "{turned:?}");
    let said = format!("{}{later}", turned.text);
    assert!(said.contains("from-the-background"), "{said}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_subagent_works_on_its_own_history_and_hands_back_its_answer() {
    let (server, bodies) = scripted_with(vec![
        calls("agent", r#"{"task":"List the files here"}"#),
        calls("glob", r#"{"pattern":"*"}"#),
        says("There are no files."),
        says("Done."),
    ])
    .await;
    let (turned, _, _) = one_turn(&server, None, "reject", None).await;
    assert_eq!(turned.text, "Done.", "{turned:?}");
    assert_eq!((turned.asked, turned.tools), (0, 2), "{turned:?}");
    let bodies = bodies.lock().unwrap();
    assert_eq!(bodies.len(), 4);
    assert!(bodies[1].contains("List the files here") && !bodies[1].contains("\"Go\""));
    assert!(bodies[3].contains("There are no files."));
}
