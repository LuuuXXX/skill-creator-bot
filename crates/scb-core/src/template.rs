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
    std::fs::create_dir_all(&skill_dir)?;
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
    let canonical_dir = skill_dir
        .canonicalize()
        .unwrap_or_else(|_| skill_dir.clone());
    let project = ProjectConfig::new(skill_name, canonical_dir);
    project.save()?;

    Ok(skill_dir)
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
