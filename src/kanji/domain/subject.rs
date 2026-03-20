use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub id: u64,
    pub characters: String,
    pub meanings: Vec<String>,
    pub readings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ReviewItem {
    pub subject: Subject,
    pub seen: usize,
    pub correct: usize,
}

impl ReviewItem {
    pub fn new(subject: Subject) -> Self {
        Self {
            subject,
            seen: 0,
            correct: 0,
        }
    }
}

impl Subject {
    pub fn from_wanikani_value(v: &Value) -> Option<Self> {
        // Expected WaniKani subject shape: { id, data: { characters, meanings[], readings[] } }
        let id = v.get("id")?.as_u64()?;
        let data = v.get("data")?;
        let characters = data.get("characters").and_then(|c| c.as_str()).unwrap_or("").to_string();
        // collect meanings, prefer primary meaning first
        let mut meanings = Vec::new();
        if let Some(mv) = data.get("meanings").and_then(|m| m.as_array()) {
            // try to find primary meaning
            if let Some(primary) = mv.iter().find_map(|m| {
                if m.get("primary").and_then(|p| p.as_bool()).unwrap_or(false) {
                    m.get("meaning").and_then(|s| s.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            }) {
                meanings.push(primary);
            }
            // then add the rest
            for m in mv.iter() {
                if let Some(mean) = m.get("meaning").and_then(|s| s.as_str()) {
                    let mstr = mean.to_string();
                    if !meanings.contains(&mstr) {
                        meanings.push(mstr);
                    }
                }
            }
        }

        let mut readings = Vec::new();
        if let Some(rv) = data.get("readings").and_then(|r| r.as_array()) {
            for r in rv.iter() {
                if let Some(read) = r.get("reading").and_then(|s| s.as_str()) {
                    readings.push(read.to_string());
                }
            }
        }

        Some(Self { id, characters, meanings, readings })
    }
}
