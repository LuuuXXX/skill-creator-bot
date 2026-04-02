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

    if !content.starts_with("---") {
        result.error(ValidationIssue::FrontmatterMissing);
        return;
    }

    // Skip past the opening `---` (3 chars) and then look for a line that is exactly `---`.
    let after_open = &content[3..];
    // The closing delimiter must be `---` on its own line.
    let end = after_open
        .find("\n---\n")
        .or_else(|| {
            // Handle file ending without trailing newline after closing `---`.
            after_open
                .find("\n---")
                .filter(|&pos| after_open[pos + 4..].is_empty())
        });

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
    fn test_valid_skill_passes() {
        let dir = std::env::temp_dir().join("scb_test_valid");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        create_valid_skill(&dir);
        let result = validate_skill(&dir).unwrap();
        assert!(result.is_valid(), "Expected valid, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_missing_skill_md_fails() {
        let dir = std::env::temp_dir().join("scb_test_missing_md");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let result = validate_skill(&dir).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(e, ValidationIssue::SkillMdNotFound)));
    }

    #[test]
    fn test_missing_frontmatter_fails() {
        let dir = std::env::temp_dir().join("scb_test_no_fm");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("SKILL.md"), "# No frontmatter here\n").unwrap();
        let result = validate_skill(&dir).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(e, ValidationIssue::FrontmatterMissing)));
    }

    #[test]
    fn test_crlf_skill_passes() {
        let dir = std::env::temp_dir().join("scb_test_crlf");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("evals")).unwrap();
        // Simulate a CRLF file (Windows line endings).
        let crlf = "---\r\nname: test\r\ndescription: A crlf skill\r\nversion: 0.1.0\r\n---\r\n# test\r\n";
        fs::write(dir.join("SKILL.md"), crlf).unwrap();
        let evals = r#"{"version":"1","evals":[]}"#;
        fs::write(dir.join("evals").join("evals.json"), evals).unwrap();
        let result = validate_skill(&dir).unwrap();
        assert!(result.is_valid(), "CRLF file should be valid, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_no_false_positive_on_similar_keys() {
        let dir = std::env::temp_dir().join("scb_test_similar_keys");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("evals")).unwrap();
        // "username" contains "name" — should still fail because "name" key is absent.
        let content = "---\nusername: test\ndescription: some desc\nversion: 0.1.0\n---\n# test\n";
        fs::write(dir.join("SKILL.md"), content).unwrap();
        let evals = r#"{"version":"1","evals":[]}"#;
        fs::write(dir.join("evals").join("evals.json"), evals).unwrap();
        let result = validate_skill(&dir).unwrap();
        assert!(!result.is_valid(), "Should report missing 'name' key");
        assert!(result.errors.iter().any(|e| matches!(e, ValidationIssue::FrontmatterMissingName)));
    }
}
