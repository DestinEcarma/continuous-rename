use anyhow::Result;
use clap::Parser;
use std::fs;

#[derive(clap::Parser)]
#[command(version, about)]
struct Cli {
    /// The directory where the files will be renamed.
    pub target: std::ffi::OsString,

    /// The pattern to rename the files.
    ///
    /// Use `{}` to insert a number at a desired location.
    /// Example: `file-{}` will be renamed to `file-1.*`, `file-2.*`, etc.
    ///
    /// If no pattern is provided, the file name will simply be the number.
    /// Example: `1.*`, `2.*`, etc.
    pub pattern: Option<String>,

    /// Accept all rename query request.
    #[arg(short, long)]
    pub yes: bool,

    /// Perform a dry run without renaming files.
    #[arg(long)]
    pub dry_run: bool,
}

/// Prompt the user for confirmation.
fn confirm() -> bool {
    dialoguer::Confirm::new()
        .with_prompt("Do you wish to proceed?")
        .default(true)
        .interact()
        .unwrap_or(false)
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let pattern = args.pattern.unwrap_or_default();

    // Create a regex pattern for a file name that already follows the pattern
    let regex = regex::Regex::new(&format!(
        "^{}.*$",
        match pattern.contains("{}") {
            true => pattern.replace("{}", r"(\d+)"),
            false => format!(r"{}(\d+)", pattern),
        }
    ))?;

    // Create a list of files to rename
    let mut to_rename = Vec::<std::path::PathBuf>::new();

    // Create a list of used numbers
    let mut used = std::collections::BTreeSet::<usize>::new();
    let mut count = 1usize;

    // First, check if there are files that already follow the pattern
    for entry in fs::read_dir(args.target)? {
        // Check if the entry is valid
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            // Check if the file name already follows the pattern
            if let Some(file_name) = path.file_name().and_then(|value| value.to_str()) {
                if regex.is_match(file_name) {
                    let capture = regex.captures(file_name).unwrap();
                    let number = capture.get(1).unwrap().as_str().parse::<usize>().unwrap();

                    used.insert(number);
                    println!("Skipping {}", path.display());
                    continue;
                }
            }

            to_rename.push(path);
        }
    }

    to_rename.sort();

    for path in to_rename {
        if path.is_file() {
            // Find the next available number
            while used.contains(&count) {
                count += 1;
            }

            let new_path = {
                let name = match pattern.contains("{}") {
                    true => pattern.replace("{}", &count.to_string()),
                    false => format!("{}{}", pattern, count),
                };

                let ext = path
                    .extension()
                    .and_then(|value| value.to_str())
                    .map_or(Default::default(), |value| format!(".{}", value));

                path.with_file_name(format!("{}{}", name, ext))
            };

            if args.dry_run {
                println!("[Dry Run] {} -> {}", path.display(), new_path.display());
                count += 1;

                continue;
            }

            println!("{} -> {}", path.display(), new_path.display());

            if args.yes || confirm() {
                match fs::rename(&path, &new_path) {
                    Ok(_) => count += 1,
                    Err(e) => eprintln!("Failed to rename: {}", e),
                }
            }
        }
    }

    Ok(())
}
