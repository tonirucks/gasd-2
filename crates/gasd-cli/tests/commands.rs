use std::{
    path::Path,
    process::{Command, Output},
};

use serde_json::Value;

fn invoke(path: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gasd"))
        .env("GASD_DATABASE", path)
        .args(args)
        .output()
        .unwrap()
}

fn json(path: &Path, args: &[&str]) -> Value {
    let mut args = args.to_vec();
    args.push("--json");
    let output = invoke(path, &args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn command_workflow_persists_and_preserves_unspecified_fields() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("nested/tasks.sqlite3");
    assert_eq!(json(&db, &["list"]), serde_json::json!([]));
    assert_eq!(json(&db, &["add", "section", "Personal"])[0]["id"], 1);
    json(&db, &["add", "section", "Work"]);
    let task = json(&db, &["add", "task", "Read", "--section", "1"]);
    assert_eq!(task[0]["title"], "Read");
    assert_eq!(task[0]["done"], false);
    let task = json(&db, &["update", "task", "1", "--done"]);
    assert_eq!(task[0]["done"], true);
    assert_eq!(task[0]["title"], "Read");
    assert_eq!(task[0]["sectionId"], 1);
    assert_eq!(
        json(&db, &["list", "tasks", "--active"]),
        serde_json::json!([])
    );
    assert_eq!(
        json(&db, &["list", "tasks", "--done"])
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let task = json(
        &db,
        &[
            "update",
            "task",
            "1",
            "--active",
            "--title",
            "Write",
            "--section",
            "2",
        ],
    );
    assert_eq!(task[0]["done"], false);
    assert_eq!(task[0]["title"], "Write");
    assert_eq!(task[0]["sectionId"], 2);
    assert_eq!(
        json(&db, &["list", "tasks", "--section", "1"]),
        serde_json::json!([])
    );
    json(&db, &["update", "section", "2", "--name", "Office"]);
    assert_eq!(json(&db, &["list", "sections"])[1]["name"], "Office");
}

#[test]
fn invalid_commands_fail_without_changes() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("db.sqlite3");
    json(&db, &["add", "section", "Personal"]);
    json(&db, &["add", "task", "Read", "--section", "1"]);
    for args in [
        vec!["update", "task", "1"],
        vec!["update", "task", "1", "--done", "--active"],
        vec!["update", "task", "999", "--done"],
        vec!["update", "task", "1", "--title", " "],
        vec!["update", "task", "1", "--section", "999"],
        vec!["add", "section", " "],
        vec!["add", "task", "Missing section"],
        vec!["list", "tasks", "--done", "--active"],
        vec!["list", "tasks", "--section", "999"],
    ] {
        let output = invoke(&db, &args);
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    let task = json(&db, &["list"]);
    assert_eq!(task[0]["title"], "Read");
    assert_eq!(task[0]["done"], false);
    assert_eq!(task[0]["sectionId"], 1);
}

#[test]
fn explicit_path_overrides_shared_environment_and_help_does_not_open_database() {
    let dir = tempfile::tempdir().unwrap();
    let env_db = dir.path().join("unused.sqlite3");
    let explicit = dir.path().join("explicit.sqlite3");
    assert!(invoke(&env_db, &["--help"]).status.success());
    assert!(invoke(&env_db, &["--version"]).status.success());
    assert!(!env_db.exists());
    let output = invoke(
        &env_db,
        &[
            "--database",
            explicit.to_str().unwrap(),
            "add",
            "section",
            "Explicit",
        ],
    );
    assert!(output.status.success());
    assert!(explicit.exists());
    assert!(!env_db.exists());
    assert_eq!(
        json(&explicit, &["list", "sections"])[0]["name"],
        "Explicit"
    );
}

#[test]
fn human_output_contains_ids_and_state() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("db.sqlite3");
    json(&db, &["add", "section", "Personal"]);
    let output = invoke(&db, &["add", "task", "Read\nnow", "--section", "1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ID\tSECTION\tSTATE\tTITLE"));
    assert!(stdout.contains("1\t1\tactive\tRead\\nnow"));
}
