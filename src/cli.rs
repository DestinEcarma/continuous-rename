use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// The directory where the files will be renamed.
    pub target: std::ffi::OsString,

    /// The pattern to rename the files.
    ///
    /// Use `{}` to insert a number at a desired location.
    /// Example: `file-{}` → `file-1.*`, `file-2.*`, etc.
    ///
    /// Use `{n:WIDTH}` for zero-padded numbers (e.g., `{n:03}` → `001`, `002`).
    ///
    /// If omitted, files are simply numbered: `1.*`, `2.*`, etc.
    pub pattern: Option<String>,

    /// Accept all rename requests without prompting.
    #[arg(short, long)]
    pub yes: bool,

    /// Preview renames without applying them.
    #[arg(long)]
    pub dry_run: bool,
}
