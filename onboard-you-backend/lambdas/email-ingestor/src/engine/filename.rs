/// Inject a UTC timestamp suffix into a filename so S3 keys never collide.
///
/// Examples:
/// - `roster.csv`  → `roster_20250425T120000Z.csv`
/// - `data`        → `data_20250425T120000Z`
pub fn timestamped(filename: &str) -> String {
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    match filename.rfind('.') {
        Some(pos) if pos > 0 => {
            let stem = &filename[..pos];
            let ext = &filename[pos..];
            format!("{stem}_{ts}{ext}")
        }
        _ => format!("{filename}_{ts}"),
    }
}

/// Return the stem of a filename (everything before the last `.`).
pub fn stem_of(filename: &str) -> &str {
    match filename.rfind('.') {
        Some(pos) if pos > 0 => &filename[..pos],
        _ => filename,
    }
}

/// Returns `true` when the filename has a `.csv` extension (case-insensitive).
pub fn is_csv(filename: &str) -> bool {
    filename.to_lowercase().ends_with(".csv")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stem_of_normal_file() {
        assert_eq!(stem_of("roster.csv"), "roster");
    }

    #[test]
    fn stem_of_no_extension() {
        assert_eq!(stem_of("roster"), "roster");
    }

    #[test]
    fn stem_of_dotfile() {
        assert_eq!(stem_of(".hidden"), ".hidden");
    }

    #[test]
    fn is_csv_true_lowercase() {
        assert!(is_csv("roster.csv"));
    }

    #[test]
    fn is_csv_true_uppercase() {
        assert!(is_csv("roster.CSV"));
    }

    #[test]
    fn is_csv_false_xlsx() {
        assert!(!is_csv("roster.xlsx"));
    }

    #[test]
    fn timestamped_injects_before_extension() {
        let result = timestamped("roster.csv");
        // Has the stem, an underscore, a 16-char timestamp, then .csv
        assert!(result.starts_with("roster_"));
        assert!(result.ends_with(".csv"));
        // Timestamp part between stem_ and .csv should be 16 chars (YYYYMMDDTHHMMSSz)
        let mid = &result["roster_".len()..result.len() - ".csv".len()];
        assert_eq!(mid.len(), 16);
    }

    #[test]
    fn timestamped_no_extension() {
        let result = timestamped("data");
        assert!(result.starts_with("data_"));
        let suffix = &result["data_".len()..];
        assert_eq!(suffix.len(), 16);
    }
}
