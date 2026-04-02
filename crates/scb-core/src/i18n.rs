use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported UI languages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Lang {
    #[default]
    ZhCn,
    EnUs,
}

impl std::fmt::Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lang::ZhCn => write!(f, "zh-CN"),
            Lang::EnUs => write!(f, "en-US"),
        }
    }
}

impl std::str::FromStr for Lang {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "zh-CN" | "zh" | "zh-cn" => Ok(Lang::ZhCn),
            "en-US" | "en" | "en-us" => Ok(Lang::EnUs),
            other => Err(anyhow::anyhow!("Unknown language: {}", other)),
        }
    }
}

/// Holds translated strings for a single language.
pub struct I18n {
    messages: HashMap<String, String>,
    lang: Lang,
}

impl I18n {
    /// Load translations from the embedded locale JSON bundles.
    ///
    /// # Panics
    /// Panics if the embedded locale JSON is malformed. This catches broken locale
    /// files at startup (or in CI) rather than silently falling back to an empty map
    /// that would cause the CLI to print raw keys at runtime.
    pub fn load(lang: &Lang) -> Self {
        let raw = match lang {
            Lang::ZhCn => include_str!("../../../locales/zh-CN.json"),
            Lang::EnUs => include_str!("../../../locales/en-US.json"),
        };
        let messages: HashMap<String, String> = serde_json::from_str(raw)
            .unwrap_or_else(|e| panic!("Embedded locale JSON for '{}' is invalid: {}", lang, e));
        Self {
            messages,
            lang: lang.clone(),
        }
    }

    /// Look up a message key, falling back to the key itself if missing.
    ///
    /// The explicit shared lifetime ensures the compiler knows the return value
    /// may come from either `self.messages` or the `key` argument — both need
    /// to outlive the returned `&str`.
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.messages.get(key).map(|s| s.as_str()).unwrap_or(key)
    }

    pub fn lang(&self) -> &Lang {
        &self.lang
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lang_parse_zh() {
        assert_eq!("zh-CN".parse::<Lang>().unwrap(), Lang::ZhCn);
        assert_eq!("zh".parse::<Lang>().unwrap(), Lang::ZhCn);
    }

    #[test]
    fn test_lang_parse_en() {
        assert_eq!("en-US".parse::<Lang>().unwrap(), Lang::EnUs);
        assert_eq!("en".parse::<Lang>().unwrap(), Lang::EnUs);
    }

    #[test]
    fn test_i18n_load_zh() {
        let i18n = I18n::load(&Lang::ZhCn);
        assert!(!i18n.t("init.success").is_empty());
        assert!(i18n.t("init.success").contains("技能"));
    }

    #[test]
    fn test_i18n_load_en() {
        let i18n = I18n::load(&Lang::EnUs);
        assert!(!i18n.t("init.success").is_empty());
        assert!(i18n.t("init.success").contains("Skill"));
    }

    #[test]
    fn test_i18n_missing_key_returns_key() {
        let i18n = I18n::load(&Lang::EnUs);
        assert_eq!(i18n.t("no.such.key"), "no.such.key");
    }
}
