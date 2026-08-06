use clap::Parser;

/// Generate application code for an ABP project.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "banna",
    version,
    about = "ABP code generator by Furkan Cemal Caliskan"
)]
pub struct Cli {
    /// Generate files without merging registrations into existing source files.
    #[arg(long)]
    pub(crate) no_merge: bool,

    /// Preview generation without changing project files or running commands.
    #[arg(long)]
    pub(crate) dry_run: bool,
}
