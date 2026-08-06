use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExternalTool {
    DotNet,
    Npm,
    Abp,
}

impl ExternalTool {
    pub(crate) const fn program(self) -> &'static str {
        match self {
            Self::DotNet => "dotnet",
            Self::Npm => "npm",
            Self::Abp => "abp",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: PathBuf,
    pub env: BTreeMap<String, String>,
    pub stdin: Option<Vec<u8>>,
}

impl CommandSpec {
    pub(crate) fn new(
        current_dir: impl Into<PathBuf>,
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            current_dir: current_dir.into(),
            env: BTreeMap::new(),
            stdin: None,
        }
    }

    pub(crate) fn display(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommandOutput {
    pub status_code: Option<i32>,
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl CommandOutput {
    pub(crate) fn failure_message(&self, spec: &CommandSpec) -> String {
        const MAX_DIAGNOSTIC_BYTES: usize = 8 * 1024;
        let stderr = String::from_utf8_lossy(&self.stderr);
        let stdout = String::from_utf8_lossy(&self.stdout);
        let diagnostic = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        let diagnostic = if diagnostic.len() > MAX_DIAGNOSTIC_BYTES {
            let mut start = diagnostic.len() - MAX_DIAGNOSTIC_BYTES;
            while !diagnostic.is_char_boundary(start) {
                start += 1;
            }
            &diagnostic[start..]
        } else {
            diagnostic
        };
        let mut message = format!(
            "{} failed with exit status {} (cwd: {})",
            spec.display(),
            self.status_code.map_or_else(
                || "terminated by signal".to_owned(),
                |code| code.to_string()
            ),
            spec.current_dir.display()
        );
        if !diagnostic.is_empty() {
            message.push_str("\ncommand output:\n");
            message.push_str(diagnostic);
        }
        message
    }
}

pub(crate) trait CommandRunner: Send + Sync {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput>;

    fn preflight(&self, _tools: &[ExternalTool]) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_commands_for_logs() {
        let spec = CommandSpec::new("/tmp", "dotnet", ["ef", "database", "update"]);
        assert_eq!(spec.display(), "dotnet ef database update");
    }

    #[test]
    fn failure_messages_include_context_and_prefer_stderr() {
        let spec = CommandSpec::new("/workspace", "npm", ["install", "vue"]);
        let output = CommandOutput {
            status_code: Some(1),
            success: false,
            stdout: b"less useful".to_vec(),
            stderr: b"npm ERR! dependency conflict".to_vec(),
        };

        let message = output.failure_message(&spec);
        assert!(message.contains("npm install vue failed with exit status 1"));
        assert!(message.contains("/workspace"));
        assert!(message.contains("npm ERR! dependency conflict"));
        assert!(!message.contains("less useful"));
    }
}
