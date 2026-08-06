use serde_json::Value;
use std::fs;
use std::process::Command;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_banna-cli"))
}

#[test]
fn project_commands_use_an_isolated_index_and_emit_json() {
    let fixture = tempfile::tempdir().expect("fixture directory should exist");
    let config = fixture.path().join("config");
    let project = fixture.path().join("demo");
    fs::create_dir_all(project.join("src")).expect("project src should exist");

    let add = cli()
        .env("BANNA_CONFIG_DIR", &config)
        .args(["project", "add", "--path"])
        .arg(&project)
        .args(["--name", "Demo", "--format", "json"])
        .output()
        .expect("project add should run");
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let added: Value = serde_json::from_slice(&add.stdout).expect("add stdout should be JSON");
    assert_eq!(added["project_name"], "Demo");

    let list = cli()
        .env("BANNA_CONFIG_DIR", &config)
        .args(["project", "list", "--format", "json"])
        .output()
        .expect("project list should run");
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    let listed: Value = serde_json::from_slice(&list.stdout).expect("list stdout should be JSON");
    assert_eq!(listed.as_array().map(Vec::len), Some(1));

    let remove = cli()
        .env("BANNA_CONFIG_DIR", &config)
        .args(["project", "remove", "Demo", "--format", "json"])
        .output()
        .expect("project remove should run");
    assert!(
        remove.status.success(),
        "{}",
        String::from_utf8_lossy(&remove.stderr)
    );
    assert!(
        project.exists(),
        "removing an index entry must not delete files"
    );
}

#[test]
fn generation_keeps_json_stdout_machine_readable() {
    let fixture = tempfile::tempdir().expect("fixture directory should exist");
    let project = fixture.path().join("demo");
    fs::create_dir_all(project.join("src")).expect("project src should exist");
    let fields = fixture.path().join("fields.json");
    fs::write(
        &fields,
        r#"[{"name":"Number","type":"string","required":true,"filterable":true,"show_in_ui":true}]"#,
    )
    .expect("field fixture should be written");

    let output = cli()
        .args(["entity", "generate", "--project"])
        .arg(&project)
        .args([
            "--domain",
            "Acme.Billing",
            "--namespace",
            "Acme.Billing.Invoices",
            "--entity",
            "Invoice",
            "--fields",
        ])
        .arg(&fields)
        .args(["--no-merge", "--non-interactive", "--format", "json"])
        .output()
        .expect("generation should run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: Value =
        serde_json::from_slice(&output.stdout).expect("stdout should contain one JSON report");
    assert_eq!(report["entity"], "Invoice");
    assert!(
        report["events"]
            .as_array()
            .is_some_and(|events| !events.is_empty())
    );
    assert!(
        report["events"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|event| event["kind"] == "created")
    );
}

#[test]
fn dry_run_generation_preserves_the_project_tree() {
    let fixture = tempfile::tempdir().expect("fixture directory should exist");
    let project = fixture.path().join("demo");
    fs::create_dir_all(project.join("src")).expect("project src should exist");
    fs::create_dir(project.join("src/Acme.Billing.Web")).expect("web project should exist");
    let sentinel = project.join("src/keep.txt");
    fs::write(&sentinel, "unchanged").expect("sentinel should exist");
    let fields = fixture.path().join("fields.json");
    fs::write(
        &fields,
        r#"[{"name":"Number","type":"string","required":true,"filterable":true,"show_in_ui":true}]"#,
    )
    .expect("field fixture should be written");

    let output = cli()
        .args(["entity", "generate", "--project"])
        .arg(&project)
        .args([
            "--domain",
            "Acme.Billing",
            "--namespace",
            "Acme.Billing.Invoices",
            "--entity",
            "Invoice",
            "--fields",
        ])
        .arg(&fields)
        .args([
            "--ui",
            "vue",
            "--dry-run",
            "--non-interactive",
            "--format",
            "json",
        ])
        .output()
        .expect("dry-run generation should run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).expect("report should be JSON");
    assert_eq!(report["dry_run"], true);
    assert!(
        report["events"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|event| event["message"].as_str().is_some_and(|message| {
                message.contains("no project files or external systems will be changed")
            }))
    );
    assert_eq!(
        fs::read_to_string(&sentinel).expect("sentinel should remain readable"),
        "unchanged"
    );
    assert_eq!(
        fs::read_dir(project.join("src"))
            .expect("src should remain readable")
            .count(),
        2
    );
    assert_eq!(
        fs::read_dir(project.join("src/Acme.Billing.Web"))
            .expect("web project should remain readable")
            .count(),
        0
    );
    assert!(!project.join(".history").exists());
}

#[test]
fn inspection_and_history_commands_share_project_metadata() {
    let fixture = tempfile::tempdir().expect("fixture directory should exist");
    let project = fixture.path().join("demo");
    fs::create_dir_all(project.join("src/Acme.Billing.Domain"))
        .expect("domain project should exist");
    fs::create_dir_all(project.join("src/Acme.Billing.Web")).expect("web project should exist");
    fs::create_dir(project.join("angular")).expect("angular project should exist");
    let history_directory = project.join(".history");
    fs::create_dir(&history_directory).expect("history directory should exist");
    fs::write(
        history_directory.join("codegen_history.json"),
        r#"{
          "project_name": "Demo",
          "project_dir": "/tmp/demo",
          "theme": "basic",
          "bootstrap_override": "none",
          "mobile_ui": "none",
          "entities": [{
            "domain": "Acme.Billing",
            "name": "Invoice",
            "namespace": "Acme.Billing.Invoices",
            "fields": [{"name":"Number","type":"string","required":true,"filterable":true,"show_in_ui":true}],
            "ui_target": "razor",
            "generated_at": "2026-08-06T12:00:00+03:00"
          }]
        }"#,
    )
    .expect("history fixture should be written");
    let generated_file = project.join("src/Acme.Billing.Domain/Invoice.cs");
    fs::write(&generated_file, "public class Invoice {}").expect("generated fixture should exist");

    let inspect = cli()
        .args(["project", "inspect", "--path"])
        .arg(&project)
        .args(["--format", "json"])
        .output()
        .expect("inspection should run");
    assert!(
        inspect.status.success(),
        "{}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let inspection: Value =
        serde_json::from_slice(&inspect.stdout).expect("inspection should be JSON");
    assert_eq!(inspection["domain"], "Acme.Billing");
    assert_eq!(inspection["entity_count"], 1);
    assert_eq!(inspection["has_mvc"], true);
    assert_eq!(inspection["has_angular"], true);

    let configure = cli()
        .args(["project", "configure", "--path"])
        .arg(&project)
        .args(["--non-interactive", "--format", "json"])
        .output()
        .expect("project configuration should run");
    assert!(
        configure.status.success(),
        "{}",
        String::from_utf8_lossy(&configure.stderr)
    );
    let configuration: Value =
        serde_json::from_slice(&configure.stdout).expect("configuration should be JSON");
    assert_eq!(configuration["changed"], false);
    assert_eq!(configuration["theme"], "basic");

    let configure_preview = cli()
        .args(["project", "configure", "--path"])
        .arg(&project)
        .args([
            "--bootstrap-override",
            "nord",
            "--dry-run",
            "--non-interactive",
            "--format",
            "json",
        ])
        .output()
        .expect("project configuration preview should run");
    assert!(
        configure_preview.status.success(),
        "{}",
        String::from_utf8_lossy(&configure_preview.stderr)
    );
    let preview: Value = serde_json::from_slice(&configure_preview.stdout)
        .expect("configuration preview should be JSON");
    assert_eq!(preview["changed"], true);
    assert_eq!(preview["dry_run"], true);
    let history_after_preview: Value = serde_json::from_slice(
        &fs::read(history_directory.join("codegen_history.json"))
            .expect("history should remain readable"),
    )
    .expect("history should remain JSON");
    assert_eq!(history_after_preview["bootstrap_override"], "none");
    assert!(
        !project
            .join("src/Acme.Billing.Web/wwwroot/global-styles.css")
            .exists()
    );

    let list = cli()
        .args(["history", "list", "--project"])
        .arg(&project)
        .args(["--format", "json"])
        .output()
        .expect("history list should run");
    let entities: Value = serde_json::from_slice(&list.stdout).expect("history should be JSON");
    assert_eq!(entities.as_array().map(Vec::len), Some(1));

    let dry_regenerate = cli()
        .args(["entity", "regenerate", "--project"])
        .arg(&project)
        .args([
            "--entity",
            "Invoice",
            "--dry-run",
            "--non-interactive",
            "--format",
            "json",
        ])
        .output()
        .expect("dry-run regeneration should run");
    assert!(
        dry_regenerate.status.success(),
        "{}",
        String::from_utf8_lossy(&dry_regenerate.stderr)
    );
    let dry_report: Value =
        serde_json::from_slice(&dry_regenerate.stdout).expect("dry-run report should be JSON");
    assert_eq!(dry_report["dry_run"], true);
    let unchanged_history: Value = serde_json::from_slice(
        &fs::read(history_directory.join("codegen_history.json"))
            .expect("history should remain readable"),
    )
    .expect("history should remain JSON");
    assert_eq!(
        unchanged_history["entities"][0]["generated_at"],
        "2026-08-06T12:00:00+03:00"
    );
    assert_eq!(
        fs::read_to_string(&generated_file).expect("generated file should remain readable"),
        "public class Invoice {}"
    );

    let regenerate = cli()
        .args(["entity", "regenerate", "--project"])
        .arg(&project)
        .args([
            "--entity",
            "Invoice",
            "--no-merge",
            "--non-interactive",
            "--format",
            "json",
        ])
        .output()
        .expect("entity regeneration should run");
    assert!(
        regenerate.status.success(),
        "{}",
        String::from_utf8_lossy(&regenerate.stderr)
    );
    let regenerated: Value =
        serde_json::from_slice(&regenerate.stdout).expect("regeneration should be JSON");
    assert_eq!(regenerated["entity"], "Invoice");

    let remove = cli()
        .args(["history", "remove", "--project"])
        .arg(&project)
        .args(["--entity", "Invoice", "--format", "json"])
        .output()
        .expect("history remove should run");
    assert!(
        remove.status.success(),
        "{}",
        String::from_utf8_lossy(&remove.stderr)
    );
    assert!(
        generated_file.exists(),
        "history removal must preserve generated files"
    );
}
