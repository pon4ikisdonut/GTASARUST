use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutsceneModel {
    pub slot_flag: i32,
    pub model_name: String,
    pub anim_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutsceneSubtitle {
    pub start_ms: i32,
    pub duration_ms: i32,
    pub gxt_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutsceneRemoveItem {
    pub position: [f32; 3],
    pub object_name: String,
}

/// Parsed GTA:SA `.cut` cutscene descriptor stored inside `anim/cuts.img`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CutsceneFile {
    pub offset: [f32; 3],
    pub models: Vec<CutsceneModel>,
    pub subtitles: Vec<CutsceneSubtitle>,
    pub removes: Vec<CutsceneRemoveItem>,
    pub extra_color_id: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CutSection {
    None,
    Info,
    Model,
    Text,
    Uncompress,
    Attach,
    Remove,
    Peffect,
    Extracol,
    Motion,
}

impl CutsceneFile {
    pub fn parse(text: &str) -> Self {
        let mut out = Self::default();
        let mut sec = CutSection::None;

        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            let lower = line.to_ascii_lowercase();
            if lower == "end" {
                sec = CutSection::None;
                continue;
            }
            if sec == CutSection::None {
                sec = match lower.as_str() {
                    s if s.starts_with("info") => CutSection::Info,
                    s if s.starts_with("model") => CutSection::Model,
                    s if s.starts_with("text") => CutSection::Text,
                    s if s.starts_with("uncompress") => CutSection::Uncompress,
                    s if s.starts_with("attach") => CutSection::Attach,
                    s if s.starts_with("remove") => CutSection::Remove,
                    s if s.starts_with("peffect") => CutSection::Peffect,
                    s if s.starts_with("extracol") => CutSection::Extracol,
                    s if s.starts_with("motion") => CutSection::Motion,
                    _ => CutSection::None,
                };
                continue;
            }

            let fields: Vec<&str> = line
                .split([',', ' ', '\t'])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();

            match sec {
                CutSection::Info
                    if fields.len() >= 4 && fields[0].eq_ignore_ascii_case("offset") =>
                {
                    out.offset = [
                        fields[1].parse().unwrap_or(0.0),
                        fields[2].parse().unwrap_or(0.0),
                        fields[3].parse().unwrap_or(0.0),
                    ];
                }
                CutSection::Model if fields.len() >= 3 => {
                    out.models.push(CutsceneModel {
                        slot_flag: fields[0].parse().unwrap_or(1),
                        model_name: fields[1].to_ascii_lowercase(),
                        anim_name: fields[2].to_ascii_lowercase(),
                    });
                }
                CutSection::Text if fields.len() >= 3 => {
                    out.subtitles.push(CutsceneSubtitle {
                        start_ms: fields[0].parse().unwrap_or(0),
                        duration_ms: fields[1].parse().unwrap_or(0),
                        gxt_key: fields[2].to_ascii_uppercase(),
                    });
                }
                CutSection::Remove if fields.len() >= 4 => {
                    out.removes.push(CutsceneRemoveItem {
                        position: [
                            fields[0].parse().unwrap_or(0.0),
                            fields[1].parse().unwrap_or(0.0),
                            fields[2].parse().unwrap_or(0.0),
                        ],
                        object_name: fields[3].to_string(),
                    });
                }
                CutSection::Extracol if !fields.is_empty() => {
                    if let Ok(id) = fields[0].parse::<i32>()
                        && id > 0
                    {
                        out.extra_color_id = Some(id - 1);
                    }
                }
                _ => {}
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sa_cut_file_text_and_offset() {
        let sample = "\
info
offset -544.633 -184.793 77.403
end
model
1, csplay, csplay
1, cswoozie, cswoozie
end
text
5300,1450,BCE4w12
7462,1977,BCE4w00
end
";
        let cut = CutsceneFile::parse(sample);
        assert!((cut.offset[0] - (-544.633)).abs() < 0.01);
        assert_eq!(cut.models.len(), 2);
        assert_eq!(cut.models[0].model_name, "csplay");
        assert_eq!(cut.subtitles.len(), 2);
        assert_eq!(cut.subtitles[0].start_ms, 5300);
        assert_eq!(cut.subtitles[0].duration_ms, 1450);
        assert_eq!(cut.subtitles[0].gxt_key, "BCE4W12");
    }
}
