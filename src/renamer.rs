use crate::{pattern, prompt};
use anyhow::Result;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

pub struct RenameConfig<'a> {
    pub target: &'a Path,
    pub pattern: &'a str,
    pub yes: bool,
    pub dry_run: bool,
}

pub fn run(cfg: RenameConfig) -> Result<()> {
    let skip_re = pattern::skipping_regex(cfg.pattern)?;

    let mut to_rename: Vec<PathBuf> = Vec::new();
    let mut used: BTreeSet<usize> = BTreeSet::new();

    for entry in fs::read_dir(cfg.target)? {
        let path = entry?.path();

        if !path.is_file() {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_owned(),
            None => {
                eprintln!("Skipping non-UTF8 filename: {}", path.display());
                continue;
            }
        };

        if let Some(caps) = skip_re.captures(&file_name)
            && let Ok(n) = caps[1].parse::<usize>()
        {
            used.insert(n);
            println!("Skipping (already numbered): {}", path.display());
            continue;
        }

        to_rename.push(path);
    }

    to_rename.sort();

    let mut counter = 1usize;

    for path in &to_rename {
        while used.contains(&counter) {
            counter += 1;
        }

        let new_path = build_new_path(path, cfg.pattern, counter)?;

        if cfg.dry_run {
            println!("[Dry Run] {} -> {}", path.display(), new_path.display());
            counter += 1;
            continue;
        }

        println!("{} -> {}", path.display(), new_path.display());

        if cfg.yes || prompt::confirm() {
            match fs::rename(path, &new_path) {
                Ok(_) => counter += 1,
                Err(e) => eprintln!("Error renaming '{}': {}", path.display(), e),
            }
        }
    }

    Ok(())
}

fn build_new_path(path: &Path, pattern: &str, number: usize) -> Result<PathBuf> {
    let stem = pattern::format_filename(pattern, number)?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map_or(String::new(), |e| format!(".{}", e));

    Ok(path.with_file_name(format!("{}{}", stem, ext)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_dir(files: &[&str]) -> TempDir {
        let dir = TempDir::new().unwrap();
        for name in files {
            fs::File::create(dir.path().join(name)).unwrap();
        }
        dir
    }

    fn files_in(dir: &TempDir) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn dry_run_does_not_rename() {
        let dir = setup_dir(&["alpha.png", "beta.png"]);

        run(RenameConfig {
            target: dir.path(),
            pattern: "file-{}",
            yes: true,
            dry_run: true,
        })
        .unwrap();

        let result = files_in(&dir);
        assert!(result.contains(&"alpha.png".to_string()));
        assert!(result.contains(&"beta.png".to_string()));
    }

    #[test]
    fn renames_files_with_positional_pattern() {
        let dir = setup_dir(&["alpha.png", "beta.jpg"]);

        run(RenameConfig {
            target: dir.path(),
            pattern: "img-{}",
            yes: true,
            dry_run: false,
        })
        .unwrap();

        let result = files_in(&dir);
        assert!(result.contains(&"img-1.png".to_string()));
        assert!(result.contains(&"img-2.jpg".to_string()));
    }

    #[test]
    fn renames_with_padded_pattern() {
        let dir = setup_dir(&["a.png", "b.png", "c.png"]);

        run(RenameConfig {
            target: dir.path(),
            pattern: "photo_{n:03}",
            yes: true,
            dry_run: false,
        })
        .unwrap();

        let result = files_in(&dir);
        assert!(result.contains(&"photo_001.png".to_string()));
        assert!(result.contains(&"photo_002.png".to_string()));
        assert!(result.contains(&"photo_003.png".to_string()));
    }

    #[test]
    fn renames_with_empty_pattern() {
        let dir = setup_dir(&["foo.txt", "bar.txt"]);

        run(RenameConfig {
            target: dir.path(),
            pattern: "",
            yes: true,
            dry_run: false,
        })
        .unwrap();

        let result = files_in(&dir);
        assert!(result.contains(&"1.txt".to_string()));
        assert!(result.contains(&"2.txt".to_string()));
    }

    #[test]
    fn skips_already_numbered_files() {
        // img-1.png already follows the pattern — should not be renamed
        let dir = setup_dir(&["img-1.png", "alpha.jpg"]);

        run(RenameConfig {
            target: dir.path(),
            pattern: "img-{}",
            yes: true,
            dry_run: false,
        })
        .unwrap();

        let result = files_in(&dir);
        assert!(
            result.contains(&"img-1.png".to_string()),
            "img-1.png should be untouched"
        );
        assert!(
            result.contains(&"img-2.jpg".to_string()),
            "alpha.jpg should get next available slot"
        );
        assert!(!result.contains(&"alpha.jpg".to_string()));
    }

    #[test]
    fn fills_gaps_in_numbering() {
        // img-1 and img-3 exist; new file should land on 2
        let dir = setup_dir(&["img-1.png", "img-3.png", "new.png"]);

        run(RenameConfig {
            target: dir.path(),
            pattern: "img-{}",
            yes: true,
            dry_run: false,
        })
        .unwrap();

        let result = files_in(&dir);
        assert!(
            result.contains(&"img-2.png".to_string()),
            "gap at 2 should be filled"
        );
    }

    #[test]
    fn preserves_extension() {
        let dir = setup_dir(&["document.pdf"]);

        run(RenameConfig {
            target: dir.path(),
            pattern: "doc-{}",
            yes: true,
            dry_run: false,
        })
        .unwrap();

        assert!(files_in(&dir).contains(&"doc-1.pdf".to_string()));
    }

    #[test]
    fn handles_no_extension() {
        let dir = setup_dir(&["README"]);

        run(RenameConfig {
            target: dir.path(),
            pattern: "file-{}",
            yes: true,
            dry_run: false,
        })
        .unwrap();

        assert!(files_in(&dir).contains(&"file-1".to_string()));
    }

    #[test]
    fn empty_directory_is_noop() {
        let dir = setup_dir(&[]);

        let result = run(RenameConfig {
            target: dir.path(),
            pattern: "file-{}",
            yes: true,
            dry_run: false,
        });

        assert!(result.is_ok());
        assert!(files_in(&dir).is_empty());
    }

    #[test]
    fn mixed_extensions_get_individual_numbers() {
        let dir = setup_dir(&["a.png", "b.jpg", "c.pdf"]);

        run(RenameConfig {
            target: dir.path(),
            pattern: "f{}",
            yes: true,
            dry_run: false,
        })
        .unwrap();

        let result = files_in(&dir);
        // Each file gets a unique number regardless of extension
        assert_eq!(result.len(), 3);
        let numbers: Vec<u32> = result
            .iter()
            .map(|n| {
                n.trim_start_matches('f')
                    .split('.')
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap()
            })
            .collect();
        let mut unique = numbers.clone();
        unique.dedup();
        assert_eq!(numbers.len(), unique.len(), "Numbers must all be unique");
    }
}
