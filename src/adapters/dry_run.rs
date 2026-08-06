use crate::ports::{CommandOutput, CommandRunner, CommandSpec};
use anyhow::Result;

/// Treats external commands as successful without starting a process.
pub(crate) struct DryRunCommandRunner;

impl CommandRunner for DryRunCommandRunner {
    fn run(&self, _spec: &CommandSpec) -> Result<CommandOutput> {
        Ok(CommandOutput {
            status_code: Some(0),
            success: true,
            stdout: Vec::new(),
            stderr: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_starts_the_requested_process() {
        let output = DryRunCommandRunner
            .run(&CommandSpec::new(
                "/missing",
                "definitely-not-a-command",
                ["x"],
            ))
            .expect("dry-run command should be simulated");
        assert!(output.success);
    }
}
