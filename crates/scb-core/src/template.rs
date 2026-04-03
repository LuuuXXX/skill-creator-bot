use crate::i18n::Lang;
use crate::schema::EvalsSchema;
use crate::project::ProjectConfig;
use std::path::Path;

/// Scaffold a new skill directory with all required files.
pub fn scaffold_skill(
    skill_name: &str,
    base_dir: &Path,
    lang: &Lang,
) -> anyhow::Result<std::path::PathBuf> {
    let skill_dir = base_dir.join(skill_name);
    // Ensure the base directory exists; this is safe and idempotent.
    // We still use create_dir (not create_dir_all) for the skill root so we
    // get a clear AlreadyExists error if the directory already exists,
    // preserving the "non-destructive" contract. Propagate the raw
    // std::io::Error so callers can downcast and localize it.
    std::fs::create_dir_all(base_dir)?;
    std::fs::create_dir(&skill_dir)?;
    std::fs::create_dir_all(skill_dir.join("evals"))?;
    std::fs::create_dir_all(skill_dir.join("scripts"))?;
    std::fs::create_dir_all(skill_dir.join("references"))?;
    std::fs::create_dir_all(skill_dir.join("assets"))?;

    // Generate SKILL.md
    let skill_md = generate_skill_md(skill_name, lang);
    std::fs::write(skill_dir.join("SKILL.md"), skill_md)?;

    // Generate evals/evals.json
    let evals = EvalsSchema::default();
    let evals_json = serde_json::to_string_pretty(&evals)?;
    std::fs::write(skill_dir.join("evals").join("evals.json"), evals_json)?;

    // Generate scb.project.json — canonicalize so the stored path is absolute
    // and independent of the working directory from which scb was invoked.
    // The directory was just created, so canonicalize should always succeed;
    // we fail fast rather than silently falling back to a relative path.
    let canonical_dir = skill_dir.canonicalize()?;
    let mut project = ProjectConfig::new(skill_name, canonical_dir.clone());
    // Persist the language in project config for future use.
    project.lang = lang.clone();
    project.save()?;

    Ok(canonical_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Lang;

    #[test]
    fn test_scaffold_creates_expected_files() {
        let tmp = tempfile::tempdir().unwrap();
        let created = scaffold_skill("my-skill", tmp.path(), &Lang::EnUs).unwrap();

        assert!(created.is_absolute(), "returned path should be absolute");
        assert!(created.join("SKILL.md").exists(), "SKILL.md should exist");
        assert!(
            created.join("evals").join("evals.json").exists(),
            "evals/evals.json should exist"
        );
        assert!(
            created.join("scb.project.json").exists(),
            "scb.project.json should exist"
        );
        assert!(created.join("scripts").exists(), "scripts/ dir should exist");
        assert!(created.join("assets").exists(), "assets/ dir should exist");
        assert!(
            created.join("references").exists(),
            "references/ dir should exist"
        );
    }

    #[test]
    fn test_scaffold_skill_md_contains_name() {
        let tmp = tempfile::tempdir().unwrap();
        let created = scaffold_skill("hello-world", tmp.path(), &Lang::EnUs).unwrap();
        let content = std::fs::read_to_string(created.join("SKILL.md")).unwrap();
        assert!(content.contains("hello-world"));
        assert!(content.contains("name: hello-world"));
    }

    #[test]
    fn test_scaffold_evals_json_valid_schema() {
        let tmp = tempfile::tempdir().unwrap();
        let created = scaffold_skill("test-skill", tmp.path(), &Lang::EnUs).unwrap();
        let schema = crate::schema::EvalsSchema::load(
            &created.join("evals").join("evals.json"),
        )
        .unwrap();
        assert_eq!(schema.version, "1");
        assert!(!schema.evals.is_empty(), "default evals should have placeholder items");
    }

    #[test]
    fn test_scaffold_project_json_has_correct_skill_name() {
        let tmp = tempfile::tempdir().unwrap();
        let created = scaffold_skill("my-skill", tmp.path(), &Lang::ZhCn).unwrap();
        let config = crate::project::ProjectConfig::load(&created).unwrap();
        assert_eq!(config.skill_name, "my-skill");
        assert_eq!(config.lang, Lang::ZhCn);
    }

    #[test]
    fn test_scaffold_fails_if_skill_dir_already_exists() {
        let tmp = tempfile::tempdir().unwrap();
        scaffold_skill("dupe-skill", tmp.path(), &Lang::EnUs).unwrap();
        let err = scaffold_skill("dupe-skill", tmp.path(), &Lang::EnUs).unwrap_err();
        let io_err = err.downcast_ref::<std::io::Error>().unwrap();
        assert_eq!(io_err.kind(), std::io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn test_scaffold_zh_cn_skill_md_contains_name() {
        let tmp = tempfile::tempdir().unwrap();
        let created = scaffold_skill("技能名称", tmp.path(), &Lang::ZhCn).unwrap();
        let content = std::fs::read_to_string(created.join("SKILL.md")).unwrap();
        assert!(content.contains("技能名称"));
        assert!(content.contains("name: 技能名称"));
    }
}

fn generate_skill_md(skill_name: &str, lang: &Lang) -> String {
    match lang {
        Lang::ZhCn => format!(
            r#"---
name: {skill_name}
description: |
  在这里简要描述该技能的功能（用于触发匹配，建议简洁且准确）。
compatibility: claude-3-5-sonnet, claude-3-7-sonnet
version: 0.1.0
---

# {skill_name}

## 概述

在这里描述技能的主要功能和使用场景。

## 工作流程

1. 接收用户请求
2. 处理请求
3. 输出结果

## 输出格式

描述技能期望的输出格式。

## 注意事项

- 注意事项 1
- 注意事项 2

## 示例

**输入：**
```
示例输入
```

**输出：**
```
示例输出
```
"#,
            skill_name = skill_name
        ),
        Lang::EnUs => format!(
            r#"---
name: {skill_name}
description: |
  Briefly describe what this skill does (used for trigger matching; keep it concise and accurate).
compatibility: claude-3-5-sonnet, claude-3-7-sonnet
version: 0.1.0
---

# {skill_name}

## Overview

Describe the main function and use cases of this skill here.

## Workflow

1. Receive user request
2. Process the request
3. Produce output

## Output Format

Describe the expected output format for this skill.

## Notes

- Note 1
- Note 2

## Examples

**Input:**
```
Example input
```

**Output:**
```
Example output
```
"#,
            skill_name = skill_name
        ),
    }
}
