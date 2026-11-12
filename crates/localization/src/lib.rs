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

    /// Parse San Andreas inline color/newline tokens (`~r~`, `~g~`, `~b~`, `~y~`, `~p~`, `~w~`, `~s~`, `~l~`, `~h~`, `~n~`)
    /// into colored text spans matching `CHudColours` from GTA: San Andreas.
    pub fn parse_sa_rich_text(input: &str, default_color: [u8; 3]) -> Vec<SaTextSpan> {
        let mut spans: Vec<SaTextSpan> = Vec::new();
        let mut cur_color = default_color;
        let mut buf = String::new();

        let flush = |spans: &mut Vec<SaTextSpan>, buf: &mut String, color: [u8; 3]| {
            if !buf.is_empty() {
                spans.push(SaTextSpan {
                    text: std::mem::take(buf),
                    color,
                });
            }
        };

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
                match tag.as_str() {
                    "n" | "N" => buf.push('\n'),
                    "r" | "R" => {
                        flush(&mut spans, &mut buf, cur_color);
                        cur_color = [180, 25, 29];
                    }
                    "g" | "G" => {
                        flush(&mut spans, &mut buf, cur_color);
                        cur_color = [54, 104, 44];
                    }
                    "b" | "B" => {
                        flush(&mut spans, &mut buf, cur_color);
                        cur_color = [50, 60, 127];
                    }
                    "y" | "Y" => {
                        flush(&mut spans, &mut buf, cur_color);
                        cur_color = [226, 192, 99];
                    }
                    "p" | "P" => {
                        flush(&mut spans, &mut buf, cur_color);
                        cur_color = [168, 110, 252];
                    }
                    "w" | "W" | "s" | "S" => {
                        flush(&mut spans, &mut buf, cur_color);
                        cur_color = default_color;
                    }
                    "l" => {
                        flush(&mut spans, &mut buf, cur_color);
                        cur_color = [0, 0, 0];
                    }
                    "h" | "H" => {
                        flush(&mut spans, &mut buf, cur_color);
                        cur_color = [
                            (f32::from(cur_color[0]) * 1.5).min(255.0) as u8,
                            (f32::from(cur_color[1]) * 1.5).min(255.0) as u8,
                            (f32::from(cur_color[2]) * 1.5).min(255.0) as u8,
                        ];
                    }
                    _ => {}
                }
            } else {
                buf.push(c);
            }
        }
        flush(&mut spans, &mut buf, cur_color);
        spans
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

/// Colored text span extracted from a GTA:SA `.gxt` formatted string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaTextSpan {
    pub text: String,
    pub color: [u8; 3],
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

    #[test]
    fn parses_sa_color_tokens_into_spans() {
        let spans = LocalizationTable::parse_sa_rich_text(
            "~r~Ballas ~w~invaded ~g~Grove Street~n~Stop them!",
            [225, 225, 225],
        );
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].text, "Ballas ");
        assert_eq!(spans[0].color, [180, 25, 29]);
        assert_eq!(spans[1].text, "invaded ");
        assert_eq!(spans[1].color, [225, 225, 225]);
        assert_eq!(spans[2].text, "Grove Street\nStop them!");
        assert_eq!(spans[2].color, [54, 104, 44]);
    }
}
