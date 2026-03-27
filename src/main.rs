mod cli;
mod pattern;
mod prompt;
mod renamer;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    renamer::run(renamer::RenameConfig {
        target: std::path::Path::new(&args.target),
        pattern: args.pattern.as_deref().unwrap_or(""),
        yes: args.yes,
        dry_run: args.dry_run,
    })
}
