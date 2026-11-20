use opsis_store::db::{DB_FILE, Database};

#[tokio::test]
async fn agent_db_and_the_app_database_share_a_folder_and_nothing_else() {
    let user_data = tempfile::tempdir().unwrap();
    let app = Database::open(&user_data.path().join(DB_FILE))
        .await
        .unwrap();
    let agent = opsis_agent::db::open(user_data.path()).await.unwrap();
    assert!(
        user_data
            .path()
            .join(opsis_agent::db::AGENT_DB_FILE)
            .is_file()
    );
    assert_eq!(
        opsis_persistence::db::table_names(&agent).await.unwrap(),
        ["allowed", "messages", "meta", "sessions", "settings"]
    );
    let app_tables = app.table_names().await.unwrap();
    assert!(
        !app_tables
            .iter()
            .any(|t| t == "sessions" || t == "messages")
    );
    assert_eq!(
        opsis_persistence::db::stored_schema_hash(&agent).await,
        Some(opsis_persistence::db::schema_hash(
            opsis_agent::db::SCHEMA_SQL
        ))
    );
}
