use std::path::Path;

/// Validation result for a skill directory.
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }

    pub fn warn(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }
}

/// Validate the skill directory structure and content.
pub fn validate_skill(skill_dir: &Path) -> anyhow::Result<ValidationResult> {
    let mut result = ValidationResult::default();

    // Check SKILL.md exists
    let skill_md = skill_dir.join("SKILL.md");
    if !skill_md.exists() {
        result.error("SKILL.md not found in skill directory");
        return Ok(result);
    }

    // Parse SKILL.md frontmatter
    let content = std::fs::read_to_string(&skill_md)?;
    validate_frontmatter(&content, &mut result);

    // Check evals/evals.json
    let evals_json = skill_dir.join("evals").join("evals.json");
    if !evals_json.exists() {
        result.warn("evals/evals.json not found (recommended)");
    } else {
        match crate::schema::EvalsSchema::load(&evals_json) {
            Ok(schema) => {
                if schema.evals.is_empty() {
                    result.warn("evals/evals.json has no eval items");
                }
            }
            Err(e) => result.error(format!("evals/evals.json is invalid: {}", e)),
        }
    }

    // Check scb.project.json
    let project_json = skill_dir.join("scb.project.json");
    if !project_json.exists() {
        result.warn("scb.project.json not found (run `scb init` to create it)");
    }

    Ok(result)
}

fn validate_frontmatter(content: &str, result: &mut ValidationResult) {
    if !content.starts_with("---") {
        result.error("SKILL.md is missing YAML frontmatter (must start with '---')");
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
            result.error("SKILL.md frontmatter is not closed with '---'");
            return;
        }
    };
    let frontmatter = &after_open[..end];

    // Check required fields
    if !frontmatter.contains("name:") {
        result.error("SKILL.md frontmatter is missing 'name' field");
    }
    if !frontmatter.contains("description:") {
        result.error("SKILL.md frontmatter is missing 'description' field");
    }

    // Warn about optional but recommended fields
    if !frontmatter.contains("version:") {
        result.warn("SKILL.md frontmatter is missing 'version' field (recommended)");
    }
    if !frontmatter.contains("compatibility:") {
        result.warn("SKILL.md frontmatter is missing 'compatibility' field (recommended)");
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
        assert!(result.errors.iter().any(|e| e.contains("SKILL.md")));
    }

    #[test]
    fn test_missing_frontmatter_fails() {
        let dir = std::env::temp_dir().join("scb_test_no_fm");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("SKILL.md"), "# No frontmatter here\n").unwrap();
        let result = validate_skill(&dir).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("frontmatter")));
    }
}
