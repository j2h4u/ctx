use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{json, Value};
use std::fs;
#[cfg(unix)]
use std::time::Duration;
use tempfile::{Builder, TempDir};
use uuid::Uuid;

fn tempdir() -> TempDir {
    let root = std::env::current_dir().unwrap().join("target/test-data");
    fs::create_dir_all(&root).unwrap();
    Builder::new().prefix("ctx-test-").tempdir_in(root).unwrap()
}

fn ctx(temp: &TempDir) -> Command {
    let mut command = Command::cargo_bin("ctx").unwrap();
    command.env("CTX_DATA_ROOT", temp.path());
    command
}

fn record(temp: &TempDir, title: &str, body: &str, tags: &[&str]) -> Value {
    let mut command = ctx(temp);
    command.args(["record", "--title", title, "--body", body, "--json"]);
    for tag in tags {
        command.args(["--tag", tag]);
    }
    let output = command.assert().success().get_output().stdout.clone();
    serde_json::from_slice::<Value>(&output).unwrap()["record"].clone()
}

fn json_output(command: &mut Command) -> Value {
    let output = command.assert().success().get_output().stdout.clone();
    serde_json::from_slice(&output).unwrap()
}

fn write_json(temp: &TempDir, name: &str, value: &Value) -> String {
    let path = temp.path().join(name);
    fs::write(&path, serde_json::to_string_pretty(value).unwrap()).unwrap();
    path.to_str().unwrap().to_string()
}

#[test]
fn root_setup_status_schema_and_validate_work() {
    let temp = tempdir();
    ctx(&temp)
        .args(["setup"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Work Recorder workspace ready"));

    ctx(&temp)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized: true"))
        .stdout(predicate::str::contains("blob_dir:"))
        .stdout(predicate::str::contains("inbox_dir:"))
        .stdout(predicate::str::contains("device_path:"));

    ctx(&temp)
        .args(["schema"])
        .assert()
        .success()
        .stdout(predicate::str::contains("work_records"));

    ctx(&temp)
        .args(["validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn root_record_show_search_context_report_and_link_pr_work() {
    let temp = tempdir();
    let first = record(
        &temp,
        "Implement search",
        "full text enough for local notes",
        &["search"],
    );
    record(&temp, "Write report", "summarize work records", &["report"]);
    let id = first["id"].as_str().unwrap();

    ctx(&temp)
        .args(["search", "local", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\": 1"))
        .stdout(predicate::str::contains("\"results\""))
        .stdout(predicate::str::contains("\"snippet\""))
        .stdout(predicate::str::contains("Implement search"));

    ctx(&temp)
        .args(["show", id, "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\": 1"))
        .stdout(predicate::str::contains("\"record\""))
        .stdout(predicate::str::contains("Implement search"));

    ctx(&temp)
        .args([
            "link-pr",
            id,
            "https://github.com/ctxrs/ctx/pull/42",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\": 1"))
        .stdout(predicate::str::contains("pull/42"));

    ctx(&temp)
        .args(["context", "report"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Work Context"))
        .stdout(predicate::str::contains("Write report"));

    ctx(&temp)
        .args(["report", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\": 1"))
        .stdout(predicate::str::contains("\"summary\""))
        .stdout(predicate::str::contains("\"record_count\": 2"));
}

#[test]
fn evidence_run_is_recorded() {
    let temp = tempdir();
    let item = record(&temp, "Run tests", "capture command output", &["evidence"]);
    let id = item["id"].as_str().unwrap();

    ctx(&temp)
        .args(["evidence", "run", "--record", id, "rustc", "--version"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\": 1"))
        .stdout(predicate::str::contains("\"evidence\""))
        .stdout(predicate::str::contains("\"exit_code\": 0"));

    ctx(&temp)
        .args(["context", "Run", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\": 1"))
        .stdout(predicate::str::contains("\"results\""))
        .stdout(predicate::str::contains("Run tests"));
}

#[test]
fn context_json_returns_agent_packet_shape() {
    let temp = tempdir();
    let first = record(
        &temp,
        "Fix checkout retry",
        "checkout retry failed after a transient package index error",
        &["checkout"],
    );
    let id = first["id"].as_str().unwrap();

    ctx(&temp)
        .args([
            "link-pr",
            id,
            "https://github.com/ctxrs/ctx/pull/123",
            "--json",
        ])
        .assert()
        .success();

    let mut command = ctx(&temp);
    command
        .env("CTX_DASHBOARD_URL", "http://127.0.0.1:3000")
        .args(["context", "checkout retry", "--json"]);
    let packet = json_output(&mut command);

    assert_eq!(packet["schema_version"], 1);
    assert_eq!(packet["query"], "checkout retry");
    assert_eq!(packet["budget"]["max_tokens"], 12_000);
    assert!(packet["budget"]["estimated_tokens"].as_u64().unwrap() > 0);
    assert_eq!(packet["results"][0]["record_id"], id);
    assert_eq!(packet["results"][0]["title"], "Fix checkout retry");
    assert_eq!(packet["results"][0]["visibility"], "local_only");
    assert_eq!(
        packet["results"][0]["links"]["dashboard"],
        format!("http://127.0.0.1:3000/records/{id}")
    );
    assert_eq!(
        packet["results"][0]["links"]["pr"],
        "https://github.com/ctxrs/ctx/pull/123"
    );
    assert_eq!(packet["pagination"]["has_more"], false);
    assert_eq!(packet["truncation"]["truncated"], false);
}

#[test]
fn context_json_omits_dashboard_url_when_not_share_safe() {
    let temp = tempdir();
    record(&temp, "Deploy dashboard", "deploy context", &["deploy"]);

    let mut command = ctx(&temp);
    command
        .env("CTX_DASHBOARD_URL", "https://secret@example.test")
        .args(["context", "deploy", "--json"]);
    let packet = json_output(&mut command);

    assert!(packet["results"][0]["links"].get("dashboard").is_none());
}

#[test]
fn context_json_includes_why_matched_citations_and_evidence() {
    let temp = tempdir();
    let record = record(
        &temp,
        "Debug rustc failure",
        "rustc failed while checking the work recorder CLI",
        &["compiler"],
    );
    let id = record["id"].as_str().unwrap();

    ctx(&temp)
        .args([
            "evidence",
            "run",
            "--record",
            id,
            "rustc",
            "--definitely-not-a-real-rustc-flag",
        ])
        .assert()
        .failure();

    let mut command = ctx(&temp);
    command.args(["context", "rustc", "--json"]);
    let packet = json_output(&mut command);
    let result = &packet["results"][0];
    let why = result["why_matched"].as_array().unwrap();
    assert!(why.iter().any(|value| value == "title"));
    assert!(why.iter().any(|value| value == "failed_command"));
    assert!(result["citations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|citation| citation["type"] == "evidence"));
    assert_eq!(result["evidence"][0]["kind"], "manual");
    assert_eq!(result["evidence"][0]["status"], "failed");
    assert_eq!(result["evidence"][0]["freshness"], "unbound");
    assert!(packet.get("stdout").is_none());
    assert!(packet.get("stderr").is_none());
}

#[test]
fn context_json_reports_token_budget_truncation() {
    let temp = tempdir();
    for index in 0..6 {
        record(
            &temp,
            &format!("Budget context record {index}"),
            &format!(
                "budget context body {index} {}",
                "long searchable detail ".repeat(30)
            ),
            &["budget"],
        );
    }

    let mut command = ctx(&temp);
    command.args([
        "context",
        "budget",
        "--json",
        "--limit",
        "6",
        "--max-tokens",
        "90",
    ]);
    let packet = json_output(&mut command);

    assert_eq!(packet["budget"]["max_tokens"], 90);
    assert!(packet["budget"]["estimated_tokens"].as_u64().unwrap() <= 90);
    assert_eq!(packet["truncation"]["truncated"], true);
    assert!(packet["truncation"]["omitted_results"].as_u64().unwrap() > 0);
    assert_eq!(packet["truncation"]["reason"], "token_budget");
    assert_eq!(packet["pagination"]["has_more"], true);
}

#[test]
fn search_json_redacts_secret_like_snippets() {
    let temp = tempdir();
    record(
        &temp,
        "Deploy token cleanup",
        "deploy failed with token=ghp_1234567890abcdef1234567890abcdef and password=hunter2",
        &["deploy"],
    );

    let mut command = ctx(&temp);
    command.args(["search", "deploy", "--json"]);
    let packet = json_output(&mut command);
    let snippet = packet["results"][0]["snippet"].as_str().unwrap();

    assert!(snippet.contains("[REDACTED]"));
    assert!(!snippet.contains("ghp_123456"));
    assert!(!snippet.contains("hunter2"));

    let mut command = ctx(&temp);
    command.args(["context", "deploy", "--json"]);
    let packet = json_output(&mut command);
    let summary = packet["results"][0]["summary"].as_str().unwrap();

    assert!(summary.contains("[REDACTED]"));
    assert!(!summary.contains("ghp_123456"));
    assert!(!summary.contains("hunter2"));
}

#[test]
fn evidence_run_truncates_stdout_to_output_cap() {
    let temp = tempdir();

    let output = ctx(&temp)
        .args([
            "evidence",
            "run",
            "--max-output-bytes",
            "4",
            "rustc",
            "--version",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let evidence: Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(evidence["schema_version"], 1);
    assert_eq!(evidence["evidence"]["exit_code"], 0);
    assert_eq!(evidence["evidence"]["stdout"].as_str().unwrap(), "rust");
}

#[cfg(unix)]
#[test]
fn evidence_timeout_kills_descendant_process_group() {
    let temp = tempdir();

    let output = ctx(&temp)
        .timeout(Duration::from_secs(5))
        .args([
            "evidence",
            "run",
            "--timeout-seconds",
            "1",
            "sh",
            "-c",
            "sleep 10 & wait",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"exit_code\": 124"))
        .stdout(predicate::str::contains(
            "command timed out after 1 seconds",
        ))
        .stderr(predicate::str::contains("evidence command exited with 124"))
        .get_output()
        .stdout
        .clone();

    let evidence: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(evidence["schema_version"], 1);
    assert_eq!(evidence["evidence"]["exit_code"], 124);
}

#[test]
fn export_and_import_round_trip() {
    let source = tempdir();
    record(&source, "Portable record", "export me", &["archive"]);

    let archive = source.path().join("archive.json");
    ctx(&source)
        .args(["export", "--output", archive.to_str().unwrap()])
        .assert()
        .success();

    let dest = tempdir();
    ctx(&dest)
        .args(["import", "--input", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("imported 1 records"));

    ctx(&dest)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Portable record"));
}

#[test]
fn import_rejects_unsupported_archive_version() {
    let temp = tempdir();
    let archive = write_json(
        &temp,
        "unsupported.json",
        &json!({
            "version": 2,
            "records": [],
            "evidence": []
        }),
    );

    ctx(&temp)
        .args(["import", "--input", &archive])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unsupported work record archive version: 2",
        ));

    ctx(&temp)
        .args(["validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn failed_import_is_atomic() {
    let temp = tempdir();
    let record = record(&temp, "Exported", "body", &["atomic"]);
    let record_id = record["id"].as_str().unwrap();
    let archive = write_json(
        &temp,
        "bad-fk.json",
        &json!({
            "version": 1,
            "records": [record],
            "evidence": [{
                "id": Uuid::new_v4(),
                "record_id": Uuid::new_v4(),
                "command": "cargo test",
                "exit_code": 0,
                "stdout": "",
                "stderr": "",
                "started_at": "2026-01-01T00:00:00Z",
                "duration_ms": 1
            }]
        }),
    );

    let dest = tempdir();
    ctx(&dest)
        .args(["import", "--input", &archive])
        .assert()
        .failure()
        .stderr(predicate::str::contains("FOREIGN KEY constraint failed"));

    ctx(&dest)
        .args(["show", record_id])
        .assert()
        .failure()
        .stderr(predicate::str::contains("record not found"));
}

#[test]
fn import_rejects_conflicts_and_overwrites_when_explicit() {
    let temp = tempdir();
    let mut record = record(&temp, "Original", "body", &["conflict"]);
    let archive = write_json(
        &temp,
        "original.json",
        &json!({
            "version": 1,
            "records": [record.clone()],
            "evidence": []
        }),
    );

    let dest = tempdir();
    ctx(&dest)
        .args(["import", "--input", &archive])
        .assert()
        .success();

    record["title"] = json!("Replacement");
    let replacement = write_json(
        &temp,
        "replacement.json",
        &json!({
            "version": 1,
            "records": [record.clone()],
            "evidence": []
        }),
    );

    ctx(&dest)
        .args(["import", "--input", &replacement])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "archive conflicts with existing record",
        ));
    ctx(&dest)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Original"))
        .stdout(predicate::str::contains("Replacement").not());

    ctx(&dest)
        .args(["import", "--input", &replacement, "--overwrite"])
        .assert()
        .success();
    ctx(&dest)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Replacement"));
}

#[test]
fn root_uninstall_removes_product_data() {
    let temp = tempdir();
    record(&temp, "Delete me", "body", &[]);
    ctx(&temp)
        .args(["uninstall", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"));
    ctx(&temp)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized: false"));
}

#[test]
fn nested_workspace_and_work_commands_remain_compatibility_aliases() {
    let temp = tempdir();
    ctx(&temp)
        .args(["workspace", "setup"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Work Recorder workspace ready"));
    ctx(&temp)
        .args(["workspace", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized: true"));

    let output = ctx(&temp)
        .args([
            "work",
            "record",
            "--title",
            "Nested alias",
            "--body",
            "compatibility",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let record: Value = serde_json::from_slice(&output).unwrap();
    let id = record["record"]["id"].as_str().unwrap();

    ctx(&temp)
        .args(["work", "show", id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Nested alias"));
    ctx(&temp)
        .args(["work", "validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));

    ctx(&temp)
        .args(["workspace", "uninstall", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"));
}
