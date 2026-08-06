use anyhow::Result;
use banna::{Cli, run};
use clap::Parser;

fn main() -> Result<()> {
    run(&Cli::parse())
}
