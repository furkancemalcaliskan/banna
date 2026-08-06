use crate::ports::{CommandOutput, CommandRunner, CommandSpec, ExternalTool};
use anyhow::{Result, anyhow};
use std::path::PathBuf;

pub(crate) struct MigrationRequest {
    pub project_root: PathBuf,
    pub domain: String,
    pub entity: String,
}

pub(super) fn run_ef_migration_with(
    runner: &dyn CommandRunner,
    request: &MigrationRequest,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let ef_dir = request
        .project_root
        .join("src")
        .join(format!("{}.EntityFrameworkCore", request.domain));
    if !ef_dir.exists() {
        return Err(anyhow!("EF Core project not found: {}", ef_dir.display()));
    }
    for notice in runner.preflight(&[ExternalTool::DotNet])? {
        log(&notice);
    }

    let migration_name = format!("Added_{}", request.entity);
    run_logged(
        runner,
        CommandSpec::new(
            &ef_dir,
            "dotnet",
            ["ef", "migrations", "add", &migration_name],
        ),
        log,
    )?;
    run_logged(
        runner,
        CommandSpec::new(&ef_dir, "dotnet", ["ef", "database", "update"]),
        log,
    )
}

fn run_logged(
    runner: &dyn CommandRunner,
    spec: CommandSpec,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    log(&format!(
        "$ {} (cwd: {})",
        spec.display(),
        spec.current_dir.display()
    ));
    let output = runner.run(&spec)?;
    log_output(&output, log);
    if !output.success {
        return Err(anyhow!(output.failure_message(&spec)));
    }
    Ok(())
}

fn log_output(output: &CommandOutput, log: &mut dyn FnMut(&str)) {
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        log(line);
    }
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        log(line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RecordingRunner(Mutex<Vec<CommandSpec>>);

    impl CommandRunner for RecordingRunner {
        fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
            self.0.lock().expect("recording lock").push(spec.clone());
            Ok(CommandOutput {
                status_code: Some(0),
                success: true,
                stdout: b"done".to_vec(),
                stderr: Vec::new(),
            })
        }
    }

    #[test]
    fn runs_add_and_update_in_the_ef_project() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        let ef_dir = fixture
            .path()
            .join("src")
            .join("Acme.Billing.EntityFrameworkCore");
        std::fs::create_dir_all(&ef_dir).expect("EF directory should exist");
        let request = MigrationRequest {
            project_root: fixture.path().into(),
            domain: "Acme.Billing".into(),
            entity: "Invoice".into(),
        };
        let runner = RecordingRunner::default();
        let mut messages = Vec::new();

        run_ef_migration_with(&runner, &request, &mut |message| {
            messages.push(message.to_owned());
        })
        .expect("migration should run");

        let commands = runner.0.lock().expect("recording lock");
        assert_eq!(commands.len(), 2);
        assert_eq!(
            commands[0].args,
            ["ef", "migrations", "add", "Added_Invoice"]
        );
        assert_eq!(commands[1].args, ["ef", "database", "update"]);
        assert!(messages.iter().any(|message| message == "done"));
    }
}
