use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;

static RE_N_FORMAT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{n:(\d+)\}").unwrap());
static RE_EMPTY_BRACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\}").unwrap());

pub enum PatternKind {
    Padded { width: usize },
    Positional,
    Suffix,
}

impl PatternKind {
    pub fn detect(pattern: &str) -> Result<Self> {
        if let Some(caps) = RE_N_FORMAT.captures(pattern) {
            let w: usize = caps[1].parse().context("Invalid padding width")?;
            Ok(PatternKind::Padded { width: w })
        } else if RE_EMPTY_BRACE.is_match(pattern) {
            Ok(PatternKind::Positional)
        } else {
            Ok(PatternKind::Suffix)
        }
    }
}

/// Formats `number` into `pattern` according to its kind.
pub fn format_filename(pattern: &str, number: usize) -> Result<String> {
    match PatternKind::detect(pattern)? {
        PatternKind::Padded { width } => {
            let n = format!("{:0width$}", number, width = width);
            Ok(RE_N_FORMAT.replace(pattern, n.as_str()).into_owned())
        }
        PatternKind::Positional => Ok(pattern.replace("{}", &number.to_string())),
        PatternKind::Suffix => Ok(format!("{}{}", pattern, number)),
    }
}

/// Builds a regex that matches filenames already following the pattern.
pub fn skipping_regex(pattern: &str) -> Result<Regex> {
    let inner = match PatternKind::detect(pattern)? {
        PatternKind::Padded { .. } => RE_N_FORMAT.replace(pattern, r"(\d+)").into_owned(),
        PatternKind::Positional => pattern.replace("{}", r"(\d+)"),
        PatternKind::Suffix => format!(r"{}(\d+)", pattern),
    };

    // Build without double-escaping the capture group
    let full = format!("^{}.*$", inner);
    Regex::new(&full)
        .with_context(|| format!("Could not build skip regex from pattern: '{}'", pattern))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod format_filename {
        use super::*;

        // {} positional placeholder
        #[test]
        fn positional_basic() {
            assert_eq!(format_filename("file-{}", 1).unwrap(), "file-1");
        }

        #[test]
        fn positional_middle() {
            assert_eq!(format_filename("img_{}_thumb", 5).unwrap(), "img_5_thumb");
        }

        #[test]
        fn positional_large_number() {
            assert_eq!(format_filename("file-{}", 9999).unwrap(), "file-9999");
        }

        #[test]
        fn positional_number_one() {
            assert_eq!(format_filename("{}", 1).unwrap(), "1");
        }

        // {n:WIDTH} padded placeholder
        #[test]
        fn padded_three_digits() {
            assert_eq!(format_filename("{n:03}", 1).unwrap(), "001");
        }

        #[test]
        fn padded_five_digits() {
            assert_eq!(format_filename("photo_{n:05}", 42).unwrap(), "photo_00042");
        }

        #[test]
        fn padded_overflow_still_works() {
            // number wider than width — should not truncate
            assert_eq!(format_filename("{n:02}", 9999).unwrap(), "9999");
        }

        #[test]
        fn padded_width_one() {
            assert_eq!(format_filename("f{n:1}", 3).unwrap(), "f3");
        }

        // suffix (no placeholder)
        #[test]
        fn suffix_basic() {
            assert_eq!(format_filename("image", 3).unwrap(), "image3");
        }

        #[test]
        fn suffix_empty_pattern() {
            assert_eq!(format_filename("", 7).unwrap(), "7");
        }

        #[test]
        fn suffix_with_dash() {
            assert_eq!(format_filename("scan-", 10).unwrap(), "scan-10");
        }
    }

    mod skipping_regex {
        use super::*;

        // positional {}
        #[test]
        fn positional_matches_numbered_file() {
            let re = skipping_regex("file-{}").unwrap();
            assert!(re.is_match("file-3.png"));
            assert!(re.is_match("file-100.jpg"));
        }

        #[test]
        fn positional_no_match_unnumbered() {
            let re = skipping_regex("file-{}").unwrap();
            assert!(!re.is_match("photo-3.png"));
            assert!(!re.is_match("file-.png")); // missing digits
        }

        #[test]
        fn positional_captures_number() {
            let re = skipping_regex("file-{}").unwrap();
            let caps = re.captures("file-42.jpg").unwrap();
            assert_eq!(&caps[1], "42");
        }

        // padded {n:WIDTH}
        #[test]
        fn padded_matches_zero_padded() {
            let re = skipping_regex("img_{n:03}").unwrap();
            assert!(re.is_match("img_001.png"));
            assert!(re.is_match("img_042.jpg"));
        }

        #[test]
        fn padded_captures_number() {
            let re = skipping_regex("{n:04}").unwrap();
            let caps = re.captures("0099.tiff").unwrap();
            assert_eq!(&caps[1], "0099");
        }

        #[test]
        fn padded_no_match_wrong_prefix() {
            let re = skipping_regex("photo_{n:03}").unwrap();
            assert!(!re.is_match("img_001.png"));
        }

        // suffix (no placeholder)
        #[test]
        fn suffix_matches_numbered() {
            let re = skipping_regex("scan").unwrap();
            assert!(re.is_match("scan1.pdf"));
            assert!(re.is_match("scan99.pdf"));
        }

        #[test]
        fn suffix_no_match_other_prefix() {
            let re = skipping_regex("scan").unwrap();
            assert!(!re.is_match("photo1.pdf"));
        }

        #[test]
        fn empty_pattern_matches_bare_number() {
            let re = skipping_regex("").unwrap();
            assert!(re.is_match("5.txt"));
            assert!(re.is_match("100.png"));
        }
    }

    mod round_trip {
        use super::*;

        fn round_trip(pattern: &str, number: usize, ext: &str) {
            let name = format_filename(pattern, number).unwrap();
            let filename = format!("{}{}", name, ext);
            let re = skipping_regex(pattern).unwrap();
            assert!(
                re.is_match(&filename),
                "Pattern '{}' formatted '{}' but skip regex didn't match it",
                pattern,
                filename
            );
            let caps = re.captures(&filename).unwrap();
            // The captured digits must parse back to the same number
            let parsed: usize = caps[1].parse().unwrap();
            assert_eq!(parsed, number);
        }

        #[test]
        fn positional_round_trip() {
            round_trip("file-{}", 1, ".png");
            round_trip("file-{}", 42, ".jpg");
            round_trip("file-{}", 1000, ".tiff");
        }

        #[test]
        fn padded_round_trip() {
            round_trip("{n:03}", 1, ".png");
            round_trip("photo_{n:05}", 42, ".jpg");
            round_trip("{n:02}", 9, "");
        }

        #[test]
        fn suffix_round_trip() {
            round_trip("scan", 1, ".pdf");
            round_trip("img", 99, ".png");
            round_trip("", 7, ".txt");
        }
    }
}
