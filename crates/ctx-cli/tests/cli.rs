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

    ctx(&temp)
        .args(["sources", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("codex"));

    ctx(&temp)
        .args([
            "import",
            "--provider",
            "codex",
            "--path",
            &fixture,
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("imported_sessions"))
        .stdout(predicate::str::contains("source_files"))
        .stdout(predicate::str::contains("source_bytes"));

    let mut list_command = ctx(&temp);
    list_command.args(["list", "--json"]);
    let listed = json_output(&mut list_command);
    let first_id = listed["items"][0]["id"].as_str().unwrap().to_owned();

    ctx(&temp)
        .args(["search", "onboarding", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("results"));

    ctx(&temp)
        .args(["search", "--file", "crates/foo/src/lib.rs", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"query\": \"\""))
        .stdout(predicate::str::contains("\"results\""));

    ctx(&temp)
        .args(["show", &first_id, "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&first_id));

    let mut context_command = ctx(&temp);
    context_command.args(["context", "onboarding", "--json"]);
    let context = json_output(&mut context_command);
    assert_eq!(context["schema_version"], 1);
    assert_eq!(context["filters"]["include_subagents"], true);
    assert!(context["results"][0].get("evidence").is_none());
    assert!(context["truncation"].get("omitted_evidence").is_none());

    ctx(&temp)
        .args(["status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("indexed_items"));

    ctx(&temp)
        .args(["doctor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"));

    ctx(&temp)
        .args(["validate", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"valid\": true"));
}
