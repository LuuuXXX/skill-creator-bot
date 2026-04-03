use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

/// Supported UI languages.
///
/// The canonical on-disk representation matches the CLI flag values (`zh-CN` /
/// `en-US`).  Lowercase aliases (`zh-cn` / `en-us`) are accepted when reading
/// so that configs written by older versions of `scb` remain valid.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum Lang {
    #[default]
    #[serde(rename = "zh-CN", alias = "zh-cn")]
    ZhCn,
    #[serde(rename = "en-US", alias = "en-us")]
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

    /// Look up a message key.
    ///
    /// Returns a borrowed `&str` from the internal message map when the key is
    /// found, or an owned copy of the key itself as a fallback.  Using
    /// `Cow<'_, str>` decouples the return-value lifetime from the `key`
    /// argument, so callers can pass temporaries (e.g. `i18n.t(&format!(…))`).
    pub fn t<'a>(&'a self, key: &str) -> Cow<'a, str> {
        match self.messages.get(key) {
            Some(val) => Cow::Borrowed(val.as_str()),
            None => Cow::Owned(key.to_owned()),
        }
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
