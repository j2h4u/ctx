use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::{fs, path::PathBuf};
use tempfile::{Builder, TempDir};

fn tempdir() -> TempDir {
    let root = std::env::current_dir().unwrap().join("target/test-data");
    fs::create_dir_all(&root).unwrap();
    Builder::new()
        .prefix("ctx-search-mvp-")
        .tempdir_in(root)
        .unwrap()
}

fn ctx(temp: &TempDir) -> Command {
    let mut command = Command::cargo_bin("ctx").unwrap();
    command.env("CTX_DATA_ROOT", temp.path());
    command
}

fn provider_history_fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/provider-history")
        .join(name)
        .to_str()
        .unwrap()
        .to_owned()
}

fn json_output(command: &mut Command) -> Value {
    let output = command.assert().success().get_output().stdout.clone();
    serde_json::from_slice(&output).unwrap()
}

fn assert_omits_keys(value: &Value, forbidden_keys: &[&str]) {
    match value {
        Value::Object(map) => {
            for key in forbidden_keys {
                assert!(
                    !map.contains_key(*key),
                    "forbidden JSON key {key} appeared in {value:#}"
                );
            }
            for nested in map.values() {
                assert_omits_keys(nested, forbidden_keys);
            }
        }
        Value::Array(items) => {
            for item in items {
                assert_omits_keys(item, forbidden_keys);
            }
        }
        _ => {}
    }
}

#[test]
fn help_exposes_only_search_mvp_commands() {
    let temp = tempdir();
    let output = ctx(&temp)
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let help = String::from_utf8(output).unwrap();
    let commands = help
        .split("Commands:")
        .nth(1)
        .and_then(|tail| tail.split("Options:").next())
        .unwrap_or(&help);

    for expected in [
        "setup", "status", "sources", "import", "list", "show", "search", "context", "doctor",
        "validate",
    ] {
        assert!(
            commands.contains(expected),
            "missing command {expected} in\n{help}"
        );
    }
    for forbidden in [
        "dashboard",
        "shim",
        "evidence",
        "publish",
        "link-pr",
        "record",
        "report",
        "export",
        "schema",
        "workspace",
        "work",
        "service",
        "capture",
        "vcs",
        "pr",
        "repair",
    ] {
        assert!(
            !commands.contains(&format!("  {forbidden}")),
            "forbidden command {forbidden} appeared in\n{help}"
        );
    }
}

#[test]
fn removed_commands_are_rejected() {
    let temp = tempdir();
    for command in [
        "dashboard",
        "shim",
        "evidence",
        "publish",
        "link-pr",
        "record",
        "report",
        "export",
        "schema",
        "workspace",
        "work",
        "service",
        "capture",
        "vcs",
        "pr",
        "repair",
    ] {
        ctx(&temp)
            .arg(command)
            .assert()
            .failure()
            .stderr(predicate::str::contains("unrecognized subcommand"));
    }
}

#[test]
fn setup_does_not_migrate_legacy_shim_directory() {
    let temp = tempdir();
    let legacy_shims = temp.path().join("work-record").join("shims");
    fs::create_dir_all(&legacy_shims).unwrap();
    fs::write(legacy_shims.join("git"), "#!/bin/sh\n").unwrap();

    ctx(&temp).arg("setup").assert().success();

    assert!(
        !temp.path().join("shims").exists(),
        "setup must not create or migrate shim directories"
    );
    assert!(
        legacy_shims.join("git").exists(),
        "legacy shim files should be left in place instead of installed"
    );
}

#[test]
fn provider_help_matches_implemented_importers() {
    let temp = tempdir();
    let output = ctx(&temp)
        .args(["import", "--help"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("[possible values: codex, pi]"));
    assert!(!help.contains("claude"));
}

#[test]
fn public_subcommand_help_is_golden_enough_for_search_mvp() {
    let temp = tempdir();
    for (command, required) in [
        ("setup", vec!["Usage: ctx setup", "--json"]),
        ("status", vec!["Usage: ctx status", "--json"]),
        ("sources", vec!["Usage: ctx sources", "--json"]),
        (
            "import",
            vec![
                "Usage: ctx import",
                "--provider <PROVIDER>",
                "[possible values: codex, pi]",
                "--path <PATH>",
                "--resume",
                "--json",
            ],
        ),
        ("list", vec!["Usage: ctx list", "--limit <LIMIT>", "--json"]),
        ("show", vec!["Usage: ctx show", "<ID>", "--json"]),
        (
            "search",
            vec![
                "Usage: ctx search",
                "[QUERY]",
                "--provider <PROVIDER>",
                "--repo <REPO>",
                "--since <SINCE>",
                "--primary-only",
                "--include-subagents",
                "--event-type <EVENT_TYPE>",
                "--file <FILE>",
                "--json",
            ],
        ),
        (
            "context",
            vec![
                "Usage: ctx context",
                "<QUERY>",
                "--max-tokens <MAX_TOKENS>",
                "--provider <PROVIDER>",
                "--repo <REPO>",
                "--since <SINCE>",
                "--primary-only",
                "--include-subagents",
                "--event-type <EVENT_TYPE>",
                "--file <FILE>",
                "--json",
            ],
        ),
        ("doctor", vec!["Usage: ctx doctor", "--json"]),
        ("validate", vec!["Usage: ctx validate", "--json"]),
    ] {
        let output = ctx(&temp)
            .args([command, "--help"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let help = String::from_utf8(output).unwrap();
        for needle in required {
            assert!(
                help.contains(needle),
                "{command} help missing {needle} in\n{help}"
            );
        }
        for forbidden in ["dashboard", "shim", "publish", "link-pr", "claude"] {
            assert!(
                !help.contains(forbidden),
                "{command} help leaked {forbidden} in\n{help}"
            );
        }
    }
}

#[test]
fn fresh_home_search_mvp_flow() {
    let temp = tempdir();
    let fixture = provider_history_fixture("codex-sessions");

    ctx(&temp)
        .arg("setup")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "local agent history search is ready",
        ));

    let setup_json = json_output(ctx(&temp).args(["setup", "--json"]));
    assert_eq!(setup_json["schema_version"], 1);
    assert_eq!(setup_json["network_required"], false);
    assert_eq!(setup_json["repo_writes"], false);

    let sources = json_output(ctx(&temp).args(["sources", "--json"]));
    assert_eq!(sources["schema_version"], 1);
    assert!(sources["sources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|source| source["provider"] == "codex"));

    let import = json_output(ctx(&temp).args([
        "import",
        "--provider",
        "codex",
        "--path",
        &fixture,
        "--json",
    ]));
    assert_eq!(import["schema_version"], 1);
    assert!(import["totals"]["imported_sessions"].as_u64().unwrap() > 0);
    assert!(import["totals"]["source_files"].as_u64().unwrap() > 0);
    assert!(import["totals"]["source_bytes"].as_u64().unwrap() > 0);

    let mut list_command = ctx(&temp);
    list_command.args(["list", "--json"]);
    let listed = json_output(&mut list_command);
    assert_eq!(listed["schema_version"], 1);
    assert_omits_keys(&listed, &["record_id", "work_record_id", "kind"]);
    assert_eq!(listed["items"][0]["item_type"], "agent_history");
    let first_id = listed["items"][0]["item_id"].as_str().unwrap().to_owned();
    assert_eq!(listed["items"][0]["id"], listed["items"][0]["item_id"]);

    let search = json_output(ctx(&temp).args(["search", "onboarding", "--json"]));
    assert_eq!(search["schema_version"], 1);
    assert_eq!(search["share_safe"], false);
    assert_omits_keys(
        &search,
        &["record_id", "work_record_id", "raw_source_path", "kind"],
    );
    assert!(search["results"][0]["item_id"].is_string());
    assert_eq!(search["results"][0]["item_type"], "agent_history");
    assert!(search["results"][0]["citations"][0]["item_id"].is_string());
    assert!(search["results"][0]["citations"][0]["item_type"].is_string());

    let file_search =
        json_output(ctx(&temp).args(["search", "--file", "crates/foo/src/lib.rs", "--json"]));
    assert_eq!(file_search["query"], "");
    assert!(file_search["results"].is_array());

    let show = json_output(ctx(&temp).args(["show", &first_id, "--json"]));
    assert_eq!(show["schema_version"], 1);
    assert_eq!(show["item"]["item_id"], first_id);
    assert_eq!(show["item"]["item_type"], "agent_history");
    assert_omits_keys(
        &show,
        &[
            "record_id",
            "work_record_id",
            "kind",
            "payload",
            "payload_blob_id",
            "dedupe_key",
            "capture_source_id",
        ],
    );
    assert!(show["events"]
        .as_array()
        .unwrap()
        .iter()
        .all(|event| event["item_type"] == "event" && event["preview"].is_string()));

    let mut context_command = ctx(&temp);
    context_command.args(["context", "onboarding", "--json"]);
    let context = json_output(&mut context_command);
    assert_eq!(context["schema_version"], 1);
    assert_eq!(context["filters"]["include_subagents"], true);
    assert_eq!(context["share_safe"], false);
    assert_omits_keys(
        &context,
        &["record_id", "work_record_id", "raw_source_path", "kind"],
    );
    assert!(context["results"][0]["item_id"].is_string());
    assert_eq!(context["results"][0]["item_type"], "agent_history");
    assert!(context["results"][0]["citations"][0]["item_id"].is_string());
    assert!(context["results"][0]["citations"][0]["item_type"].is_string());
    assert!(context["results"][0].get("evidence").is_none());
    assert!(context["truncation"].get("omitted_evidence").is_none());

    let status = json_output(ctx(&temp).args(["status", "--json"]));
    assert_eq!(status["schema_version"], 1);
    assert!(status["indexed_items"].as_u64().unwrap() > 0);

    let doctor = json_output(ctx(&temp).args(["doctor", "--json"]));
    assert_eq!(doctor["schema_version"], 1);
    assert_eq!(doctor["ok"], true);

    let validate = json_output(ctx(&temp).args(["validate", "--json"]));
    assert_eq!(validate["schema_version"], 1);
    assert_eq!(validate["valid"], true);
}
