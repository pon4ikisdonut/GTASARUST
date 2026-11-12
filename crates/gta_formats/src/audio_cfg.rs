use serde::{Deserialize, Serialize};

/// Metadata describing an external GTA:SA audio stream file (e.g., radio station or cutscene stream).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamStatus {
    pub name: String,
    pub byte_len: u64,
    pub is_present: bool,
}

impl AudioStreamStatus {
    pub fn from_metadata(name: impl Into<String>, byte_len: u64) -> Self {
        Self {
            name: name.into(),
            byte_len,
            is_present: byte_len > 0,
        }
    }
}

/// Entry parsed from `audio/CONFIG/AudioEvents.txt` or `data/AudioEvents.txt`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEventMapping {
    pub id: i32,
    pub name: String,
}

pub fn parse_audio_events(text: &str) -> Vec<AudioEventMapping> {
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.split('#').next().unwrap_or("").trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        if let (Some(a), Some(b)) = (parts.next(), parts.next())
            && let Ok(id) = b.parse::<i32>()
        {
            out.push(AudioEventMapping {
                id,
                name: a.to_string(),
            });
        }
    }
    out
}
