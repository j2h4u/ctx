mod support;

use support::*;

#[test]
fn antigravity_cli_partial_import_skips_malformed_file_among_valid_files() {
    let temp = tempdir();
    let brain = write_antigravity_valid_and_malformed_file_tree(&temp);

    let imported = json_output(ctx(&temp).args([
        "import",
        "--provider",
        "antigravity",
        "--path",
        brain.to_str().unwrap(),
        "--partial",
        "--json",
        "--progress",
        "none",
    ]));
    assert_eq!(imported["totals"]["source_files"], 2, "{imported:#}");
    assert_eq!(imported["totals"]["imported_sessions"], 1, "{imported:#}");
    assert_eq!(imported["totals"]["imported_events"], 3, "{imported:#}");
    assert_eq!(imported["totals"]["failed"], 1, "{imported:#}");
    assert_eq!(imported["totals"]["failed_sources"], 0, "{imported:#}");
    assert!(imported["sources"][0]["failures"]
        .as_array()
        .unwrap()
        .iter()
        .any(|failure| failure["error"].as_str().unwrap().contains("agy-bad")));

    let status = json_output(ctx(&temp).args(["status", "--json"]));
    assert_eq!(status["source_import_files"], 2, "{status:#}");
    assert_eq!(status["indexed_source_import_files"], 1, "{status:#}");
    assert_eq!(status["failed_source_import_files"], 1, "{status:#}");
    assert_eq!(status["pending_source_import_files"], 1, "{status:#}");

    let search = json_output(ctx(&temp).args([
        "search",
        "write_to_file",
        "--provider",
        "antigravity",
        "--json",
    ]));
    assert_search_provider_oracle(&search, "antigravity", "write_to_file", 1, "tool_call");

    let strict_temp = tempdir();
    let strict_brain = write_antigravity_valid_and_malformed_file_tree(&strict_temp);
    let stderr = failure_stderr(ctx(&strict_temp).args([
        "import",
        "--provider",
        "antigravity",
        "--path",
        strict_brain.to_str().unwrap(),
        "--json",
    ]));
    assert!(stderr.contains("failed with 1 failure"), "{stderr}");
    assert_import_store_empty_after_atomic_failure(&strict_temp);
}

#[test]
fn native_import_passes_runtime_user_to_capture_source() {
    let temp = tempdir();
    let brain = write_antigravity_valid_and_malformed_file_tree(&temp);
    fs::remove_dir_all(brain.join("agy-bad")).unwrap();

    ctx(&temp)
        .args([
            "import",
            "--provider",
            "antigravity",
            "--path",
            brain.to_str().unwrap(),
            "--runtime-user",
            "alice",
            "--no-daemon",
            "--progress",
            "none",
        ])
        .assert()
        .success();

    let conn = Connection::open(temp.path().join("work.sqlite")).unwrap();
    let runtime_user: Option<String> = conn
        .query_row("SELECT runtime_user FROM capture_sources", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(runtime_user.as_deref(), Some("alice"));
}

#[test]
fn explicit_codex_session_import_preserves_runtime_user_provenance() {
    let temp = tempdir();
    let sessions = PathBuf::from(provider_history_fixture("codex-sessions"));
    let session = sessions.join("2026/06/23/alice.jsonl");

    ctx(&temp)
        .args([
            "import",
            "--provider",
            "codex",
            "--path",
            session.to_str().unwrap(),
            "--runtime-user",
            "alice",
            "--no-daemon",
            "--progress",
            "none",
        ])
        .assert()
        .success();

    let conn = Connection::open(temp.path().join("work.sqlite")).unwrap();
    let (runtime_user, source_identity): (Option<String>, String) = conn
        .query_row(
            "SELECT runtime_user, source_identity FROM capture_sources WHERE runtime_user = 'alice'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(runtime_user.as_deref(), Some("alice"));
    assert!(source_identity.contains("alice"), "{source_identity}");
}

#[test]
fn codex_directory_import_keeps_runtime_user_observations_separate() {
    let temp = tempdir();
    let sessions = temp.path().join("codex-sessions/2026/07/13");
    fs::create_dir_all(&sessions).unwrap();
    let session = sessions.join("rollout-2026-07-13T10-00-00-runtime-user.jsonl");
    fs::write(
        &session,
        concat!(
            r#"{"timestamp":"2026-07-13T10:00:00Z","type":"session_meta","payload":{"id":"runtime-user-shared-session","timestamp":"2026-07-13T10:00:00Z","cwd":"/workspace","originator":"codex-cli"}}"#,
            "\n",
            r#"{"timestamp":"2026-07-13T10:00:01Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"*** Begin Patch\n*** Update File: src/runtime_user.rs\n@@\n-old\n+new\n*** End Patch"}]}}"#,
            "\n"
        ),
    )
    .unwrap();

    for runtime_user in ["alice", "bob"] {
        ctx(&temp)
            .args([
                "import",
                "--provider",
                "codex",
                "--path",
                sessions
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap(),
                "--runtime-user",
                runtime_user,
                "--no-daemon",
                "--progress",
                "none",
            ])
            .assert()
            .success();
    }

    let conn = Connection::open(temp.path().join("work.sqlite")).unwrap();
    for (table, expected) in [
        ("capture_sources", 2),
        ("sessions", 2),
        ("files_touched", 2),
        ("events", 1),
        ("event_observations", 2),
    ] {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, expected, "{table}");
    }
}

fn write_antigravity_valid_and_malformed_file_tree(temp: &TempDir) -> PathBuf {
    let source_fixture = PathBuf::from(provider_history_fixture("antigravity/v1/brain"));
    let brain = temp.path().join("brain");
    let valid_logs = brain
        .join("agy-success")
        .join(".system_generated")
        .join("logs");
    fs::create_dir_all(&valid_logs).unwrap();
    fs::copy(
        source_fixture
            .join("agy-success")
            .join(".system_generated")
            .join("logs")
            .join("transcript_full.jsonl"),
        valid_logs.join("transcript_full.jsonl"),
    )
    .unwrap();

    let bad_logs = brain.join("agy-bad").join(".system_generated").join("logs");
    fs::create_dir_all(&bad_logs).unwrap();
    fs::write(bad_logs.join("transcript_full.jsonl"), "{\"step_index\":\n").unwrap();
    brain
}

fn assert_import_store_empty_after_atomic_failure(temp: &TempDir) {
    let conn = Connection::open(temp.path().join("work.sqlite")).unwrap();
    for table in [
        "history_records",
        "ctx_events",
        "ctx_sessions",
        "capture_sources",
    ] {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0, "{table} should be empty after atomic failure");
    }
}
