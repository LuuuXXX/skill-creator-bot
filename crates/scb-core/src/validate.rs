use std::path::Path;

/// A structured validation issue returned by scb-core.
///
/// Each variant corresponds to a specific problem found during validation.
/// Callers (e.g. scb-cli) map variants to localized user-facing messages via
/// `ValidationIssue::i18n_key()` and optional `detail()` substitution.
#[derive(Debug, Clone)]
pub enum ValidationIssue {
    /// SKILL.md is not present in the skill directory.
    SkillMdNotFound,
    /// SKILL.md does not begin with a YAML frontmatter opening delimiter (`---`).
    FrontmatterMissing,
    /// SKILL.md frontmatter is not closed with a second `---` delimiter.
    FrontmatterNotClosed,
    /// The `name` key is absent from the frontmatter.
    FrontmatterMissingName,
    /// The `description` key is absent from the frontmatter.
    FrontmatterMissingDescription,
    /// The `version` key is absent (recommended).
    FrontmatterMissingVersion,
    /// The `compatibility` key is absent (recommended).
    FrontmatterMissingCompatibility,
    /// `evals/evals.json` is not present (recommended).
    EvalsJsonNotFound,
    /// `evals/evals.json` could not be parsed; contains the underlying error.
    EvalsJsonInvalid(String),
    /// `evals/evals.json` contains no eval items.
    EvalsJsonEmpty,
    /// `scb.project.json` is not present (recommended).
    ProjectJsonNotFound,
    /// The path provided for the skill directory exists but is not a directory.
    SkillDirNotDirectory,
}

impl ValidationIssue {
    /// Returns the i18n key used to look up the localized message for this issue.
    pub fn i18n_key(&self) -> &'static str {
        match self {
            Self::SkillMdNotFound => "validate.issue.skill_md_not_found",
            Self::FrontmatterMissing => "validate.issue.frontmatter_missing",
            Self::FrontmatterNotClosed => "validate.issue.frontmatter_not_closed",
            Self::FrontmatterMissingName => "validate.issue.frontmatter_missing_name",
            Self::FrontmatterMissingDescription => "validate.issue.frontmatter_missing_description",
            Self::FrontmatterMissingVersion => "validate.issue.frontmatter_missing_version",
            Self::FrontmatterMissingCompatibility => "validate.issue.frontmatter_missing_compatibility",
            Self::EvalsJsonNotFound => "validate.issue.evals_json_not_found",
            Self::EvalsJsonInvalid(_) => "validate.issue.evals_json_invalid",
            Self::EvalsJsonEmpty => "validate.issue.evals_json_empty",
            Self::ProjectJsonNotFound => "validate.issue.project_json_not_found",
            Self::SkillDirNotDirectory => "validate.issue.skill_dir_not_directory",
        }
    }

    /// Returns an optional detail string for issues with dynamic content.
    ///
    /// When non-`None`, callers should substitute `{detail}` in the localized
    /// message template with this value.
    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::EvalsJsonInvalid(d) => Some(d.as_str()),
            _ => None,
        }
    }
}

/// Validation result for a skill directory.
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn error(&mut self, issue: ValidationIssue) {
        self.errors.push(issue);
    }

    pub fn warn(&mut self, issue: ValidationIssue) {
        self.warnings.push(issue);
    }
}

/// Validate the skill directory structure and content.
pub fn validate_skill(skill_dir: &Path) -> anyhow::Result<ValidationResult> {
    // Verify that skill_dir exists and is a directory before checking its
    // contents.  Propagating the raw std::io::Error here (e.g. NotFound) lets
    // the CLI's validate.check_failed context fire with an accurate message
    // rather than producing a misleading SkillMdNotFound error for a path
    // that doesn't exist at all.
    let meta = std::fs::metadata(skill_dir)?;
    if !meta.is_dir() {
        let mut result = ValidationResult::default();
        result.error(ValidationIssue::SkillDirNotDirectory);
        return Ok(result);
    }

    let mut result = ValidationResult::default();

    // Check SKILL.md exists
    let skill_md = skill_dir.join("SKILL.md");
    if !skill_md.exists() {
        result.error(ValidationIssue::SkillMdNotFound);
        return Ok(result);
    }

    // Parse SKILL.md frontmatter
    let content = std::fs::read_to_string(&skill_md)?;
    validate_frontmatter(&content, &mut result);

    // Check evals/evals.json
    let evals_json = skill_dir.join("evals").join("evals.json");
    if !evals_json.exists() {
        result.warn(ValidationIssue::EvalsJsonNotFound);
    } else {
        match crate::schema::EvalsSchema::load(&evals_json) {
            Ok(schema) => {
                if schema.evals.is_empty() {
                    result.warn(ValidationIssue::EvalsJsonEmpty);
                }
            }
            Err(e) => result.error(ValidationIssue::EvalsJsonInvalid(e.to_string())),
        }
    }

    // Check scb.project.json
    let project_json = skill_dir.join("scb.project.json");
    if !project_json.exists() {
        result.warn(ValidationIssue::ProjectJsonNotFound);
    }

    Ok(result)
}

fn validate_frontmatter(content: &str, result: &mut ValidationResult) {
    // Normalize CRLF so the parser works on both Windows and Unix line endings.
    let content = content.replace("\r\n", "\n");

    // Require the opening delimiter to be exactly `---` on its own line.
    // Checking only `starts_with("---")` would also accept `---name: foo`
    // (an inline YAML document tag), which would then report FrontmatterNotClosed
    // rather than the more accurate FrontmatterMissing.
    if content == "---" {
        // File contains only the opening delimiter with no content/closing.
        result.error(ValidationIssue::FrontmatterNotClosed);
        return;
    }
    if !content.starts_with("---\n") {
        result.error(ValidationIssue::FrontmatterMissing);
        return;
    }

    // Skip past the opening `---\n` line (4 chars) and then look for a line that is exactly `---`.
    let after_open = &content[4..];
    // The closing delimiter must be `---` on its own line.
    // Special-case: `after_open` itself starts with `---` (optionally followed by `\n` or EOF)
    // — this means the frontmatter is empty (e.g. `---\n---\n` or `---\n---`).
    let end = if after_open == "---"
        || after_open.starts_with("---\n")
    {
        Some(0)
    } else {
        after_open
            .find("\n---\n")
            .or_else(|| {
                // Handle file ending without trailing newline after closing `---`.
                after_open
                    .find("\n---")
                    .filter(|&pos| after_open[pos + 4..].is_empty())
            })
    };

    let end = match end {
        Some(pos) => pos,
        None => {
            result.error(ValidationIssue::FrontmatterNotClosed);
            return;
        }
    };
    let frontmatter = &after_open[..end];

    // Check whether a YAML key is present as an actual key (not a substring of another key).
    // e.g. "username:" must not satisfy a search for "name:".
    let has_field = |key: &str| {
        frontmatter.lines().any(|line| {
            let trimmed = line.trim_start();
            if let Some((k, _rest)) = trimmed.split_once(':') {
                k.trim() == key
            } else {
                false
            }
        })
    };

    // Check required fields
    if !has_field("name") {
        result.error(ValidationIssue::FrontmatterMissingName);
    }
    if !has_field("description") {
        result.error(ValidationIssue::FrontmatterMissingDescription);
    }

    // Warn about optional but recommended fields
    if !has_field("version") {
        result.warn(ValidationIssue::FrontmatterMissingVersion);
    }
    if !has_field("compatibility") {
        result.warn(ValidationIssue::FrontmatterMissingCompatibility);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_valid_skill(dir: &std::path::Path) {
        fs::create_dir_all(dir.join("evals")).unwrap();
        fs::write(
            dir.join("SKILL.md"),
            "---\nname: test\ndescription: A test skill\nversion: 0.1.0\n---\n# test\n",
        )
        .unwrap();
        let evals = r#"{"version":"1","evals":[]}"#;
        fs::write(dir.join("evals").join("evals.json"), evals).unwrap();
    }

    #[test]
    fn test_nonexistent_dir_returns_io_error() {
        let tmp = tempfile::tempdir().unwrap();
        let nonexistent = tmp.path().join("does_not_exist");
        let err = validate_skill(&nonexistent).unwrap_err();
        // Should propagate a raw I/O error (NotFound), not produce SkillMdNotFound.
        assert!(
            err.downcast_ref::<std::io::Error>().is_some(),
            "Expected std::io::Error, got: {err}"
        );
    }

    #[test]
    fn test_not_a_directory_returns_validation_issue() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("not_a_dir.txt");
        fs::write(&file_path, "hello").unwrap();
        let result = validate_skill(&file_path).unwrap();
        assert!(!result.is_valid());
        assert!(
            result.errors.iter().any(|e| matches!(e, ValidationIssue::SkillDirNotDirectory)),
            "Expected SkillDirNotDirectory, got: {:?}", result.errors
        );
    }

    #[test]
    fn test_valid_skill_passes() {
        let tmp = tempfile::tempdir().unwrap();
        create_valid_skill(tmp.path());
        let result = validate_skill(tmp.path()).unwrap();
        assert!(result.is_valid(), "Expected valid, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_missing_skill_md_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let result = validate_skill(tmp.path()).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(e, ValidationIssue::SkillMdNotFound)));
    }

    #[test]
    fn test_missing_frontmatter_fails() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("SKILL.md"), "# No frontmatter here\n").unwrap();
        let result = validate_skill(tmp.path()).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(e, ValidationIssue::FrontmatterMissing)));
    }

    #[test]
    fn test_inline_yaml_tag_fails_as_frontmatter_missing() {
        // `---name: foo` starts with `---` but is not on its own line; should
        // report FrontmatterMissing, not FrontmatterNotClosed.
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("SKILL.md"), "---name: foo\n").unwrap();
        let result = validate_skill(tmp.path()).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(e, ValidationIssue::FrontmatterMissing)));
    }

    #[test]
    fn test_empty_frontmatter_reports_missing_fields() {
        // `---\n---\n` has valid delimiters but an empty body — should report
        // missing required fields rather than FrontmatterNotClosed.
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("SKILL.md"), "---\n---\n# body\n").unwrap();
        let result = validate_skill(tmp.path()).unwrap();
        assert!(!result.is_valid(), "Empty frontmatter should fail validation");
        assert!(
            result.errors.iter().any(|e| matches!(e, ValidationIssue::FrontmatterMissingName)),
            "Expected FrontmatterMissingName, got: {:?}", result.errors
        );
        assert!(
            !result.errors.iter().any(|e| matches!(e, ValidationIssue::FrontmatterNotClosed)),
            "Should not report FrontmatterNotClosed for empty but closed frontmatter"
        );
    }

    #[test]
    fn test_crlf_skill_passes() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("evals")).unwrap();
        // Simulate a CRLF file (Windows line endings).
        let crlf = "---\r\nname: test\r\ndescription: A crlf skill\r\nversion: 0.1.0\r\n---\r\n# test\r\n";
        fs::write(tmp.path().join("SKILL.md"), crlf).unwrap();
        let evals = r#"{"version":"1","evals":[]}"#;
        fs::write(tmp.path().join("evals").join("evals.json"), evals).unwrap();
        let result = validate_skill(tmp.path()).unwrap();
        assert!(result.is_valid(), "CRLF file should be valid, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_no_false_positive_on_similar_keys() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("evals")).unwrap();
        // "username" contains "name" — should still fail because "name" key is absent.
        let content = "---\nusername: test\ndescription: some desc\nversion: 0.1.0\n---\n# test\n";
        fs::write(tmp.path().join("SKILL.md"), content).unwrap();
        let evals = r#"{"version":"1","evals":[]}"#;
        fs::write(tmp.path().join("evals").join("evals.json"), evals).unwrap();
        let result = validate_skill(tmp.path()).unwrap();
        assert!(!result.is_valid(), "Should report missing 'name' key");
        assert!(result.errors.iter().any(|e| matches!(e, ValidationIssue::FrontmatterMissingName)));
    }
}
