use super::toolchain::{ResolvedProgram, Toolchain};
use crate::ports::{CommandOutput, CommandRunner, CommandSpec, ExternalTool};
use anyhow::{Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

pub(crate) struct ProcessCommandRunner;

impl CommandRunner for ProcessCommandRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
        let resolved = Toolchain::platform_default()
            .resolve(&spec.program)
            .with_context(|| format!("cannot prepare command '{}'", spec.display()))?;
        run_resolved(spec, resolved)
    }

    fn preflight(&self, tools: &[ExternalTool]) -> Result<Vec<String>> {
        let toolchain = Toolchain::platform_default();
        let mut notices = Vec::new();
        for tool in tools {
            let resolved = toolchain
                .resolve(tool.program())
                .with_context(|| format!("{} preflight failed", tool.program()))?;
            notices.extend(resolved.notices);
        }
        Ok(notices)
    }
}

fn run_resolved(spec: &CommandSpec, resolved: ResolvedProgram) -> Result<CommandOutput> {
    let mut command = Command::new(&resolved.executable);
    command
        .args(&resolved.prefix_args)
        .args(&spec.args)
        .current_dir(&spec.current_dir)
        .envs(&resolved.env)
        .envs(&spec.env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(if spec.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });

    let mut child = command.spawn().with_context(|| {
        format!(
            "failed to spawn {} for '{}'",
            resolved.executable.display(),
            spec.display()
        )
    })?;
    if let (Some(payload), Some(mut stdin)) = (&spec.stdin, child.stdin.take()) {
        stdin
            .write_all(payload)
            .with_context(|| format!("failed to write stdin for {}", spec.program))?;
    }

    let output = child
        .wait_with_output()
        .with_context(|| format!("failed to wait for {}", spec.program))?;
    let mut stdout = resolved.notices.join("\n").into_bytes();
    if !stdout.is_empty() {
        stdout.push(b'\n');
    }
    stdout.extend(output.stdout);
    Ok(CommandOutput {
        status_code: output.status.code(),
        success: output.status.success(),
        stdout,
        stderr: output.stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn captures_process_output_and_status() {
        let runner = ProcessCommandRunner;
        let spec = CommandSpec::new("/tmp", "sh", ["-c", "printf banna; printf warning >&2"]);
        let output = runner.run(&spec).expect("shell command should run");

        assert!(output.success);
        assert_eq!(output.stdout, b"banna");
        assert_eq!(output.stderr, b"warning");
    }

    #[cfg(windows)]
    #[test]
    fn captures_process_output_and_status() {
        let runner = ProcessCommandRunner;
        let spec = CommandSpec::new(".", "cmd", ["/C", "echo banna"]);
        let output = runner.run(&spec).expect("shell command should run");

        assert!(output.success);
        assert!(String::from_utf8_lossy(&output.stdout).contains("banna"));
    }
}
