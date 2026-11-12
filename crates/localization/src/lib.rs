//! Localization lookup backed by original GTA:SA `.gxt` tables with clean JSON fallbacks.

use gtasarust_gta_formats::gxt::GxtArchive;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct LocalizationTable {
    gxt: Option<GxtArchive>,
    fallback: HashMap<String, String>,
}

impl LocalizationTable {
    pub fn with_fallback_json(json_text: &str) -> Self {
        let fallback =
            serde_json::from_str::<HashMap<String, String>>(json_text).unwrap_or_default();
        Self {
            gxt: None,
            fallback,
        }
    }

    pub fn attach_gxt(&mut self, gxt: GxtArchive) {
        self.gxt = Some(gxt);
    }

    pub fn get<'a>(&'a self, table: &str, key: &'a str) -> &'a str {
        if let Some(gxt) = &self.gxt
            && let Some(found) = gxt.get(table, key)
        {
            return found;
        }
        self.fallback.get(key).map(String::as_str).unwrap_or(key)
    }

    /// Strip San Andreas inline color/formatting tokens (`~r~`, `~g~`, `~w~`, `~n~`) for plain text UI.
    pub fn strip_sa_tags(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut chars = input.chars();
        while let Some(c) = chars.next() {
            if c == '~' {
                let mut tag = String::new();
                for tc in chars.by_ref() {
                    if tc == '~' {
                        break;
                    }
                    tag.push(tc);
                }
                if tag == "n" || tag == "N" {
                    out.push('\n');
                }
            } else {
                out.push(c);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_sa_formatting_tags() {
        assert_eq!(
            LocalizationTable::strip_sa_tags("~r~Grove Street~w~ ~n~Home."),
            "Grove Street \nHome."
        );
    }
}
