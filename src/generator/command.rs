use crate::ports::{CommandRunner, CommandSpec, ExternalTool};
use anyhow::{Result, anyhow};
use std::path::Path;

pub(super) fn run_command_with_output(
    runner: &dyn CommandRunner,
    workdir: &Path,
    program: &str,
    args: &[&str],
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let spec = CommandSpec::new(workdir, program, args.iter().copied());
    let display = spec.display();
    log(&format!("$ {display} (cwd: {})", workdir.display()));

    let output = runner.run(&spec)?;

    if !output.stdout.is_empty() {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            log(line);
        }
    }
    if !output.stderr.is_empty() {
        for line in String::from_utf8_lossy(&output.stderr).lines() {
            log(line);
        }
    }

    if !output.success {
        return Err(anyhow!(output.failure_message(&spec)));
    }

    Ok(())
}

pub(super) fn preflight_tools(
    runner: &dyn CommandRunner,
    tools: &[ExternalTool],
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    for notice in runner.preflight(tools)? {
        log(&notice);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::CommandOutput;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RecordingRunner(Mutex<Option<CommandSpec>>);

    impl CommandRunner for RecordingRunner {
        fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
            *self.0.lock().expect("recording lock") = Some(spec.clone());
            Ok(CommandOutput {
                status_code: Some(0),
                success: true,
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
        }
    }

    #[test]
    fn executes_npm_without_a_privilege_wrapper() {
        let runner = RecordingRunner::default();
        let mut messages = Vec::new();

        run_command_with_output(
            &runner,
            Path::new("/tmp"),
            "npm",
            &["install", "vue"],
            &mut |message| messages.push(message.to_owned()),
        )
        .expect("npm command should run");

        let command = runner
            .0
            .lock()
            .expect("recording lock")
            .clone()
            .expect("command should be recorded");
        assert_eq!(command.program, "npm");
        assert_eq!(command.args, ["install", "vue"]);
        assert!(command.stdin.is_none());
        assert!(
            messages
                .iter()
                .any(|message| message.contains("$ npm install vue"))
        );
    }
}
