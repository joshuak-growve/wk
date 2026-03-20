use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use directories::ProjectDirs;
use dotenvy::dotenv;
use hex::encode as hex_encode;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, IF_MODIFIED_SINCE, IF_NONE_MATCH};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[allow(dead_code)]
#[derive(Debug)]
pub enum WanikaniError {
    MissingApiKey,
    Reqwest(reqwest::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    Other(String),
}

impl From<reqwest::Error> for WanikaniError {
    fn from(e: reqwest::Error) -> Self {
        WanikaniError::Reqwest(e)
    }
}
impl From<std::io::Error> for WanikaniError {
    fn from(e: std::io::Error) -> Self {
        WanikaniError::Io(e)
    }
}
impl From<serde_json::Error> for WanikaniError {
    fn from(e: serde_json::Error) -> Self {
        WanikaniError::Json(e)
    }
}

pub struct WaniKaniClient {
    client: Client,
    api_key: String,
    base_url: String,
    cache_dir: PathBuf,
}

#[allow(dead_code)]
impl WaniKaniClient {
    pub fn new_from_env() -> Result<Self, WanikaniError> {
        dotenv().ok();
        let key = std::env::var("WANI_KANI_API_KEY").map_err(|_| WanikaniError::MissingApiKey)?;
        Self::new(&key)
    }

    pub fn new(api_key: &str) -> Result<Self, WanikaniError> {
        let client = Client::builder().build()?;
        let base_url = "https://api.wanikani.com/v2".to_string();

        let cache_dir = if let Some(proj) = ProjectDirs::from("dev", "local", "wk") {
            proj.cache_dir().join("wanikani")
        } else {
            PathBuf::from("./.cache/wanikani")
        };
        fs::create_dir_all(&cache_dir)?;

        Ok(WaniKaniClient {
            client,
            api_key: api_key.to_string(),
            base_url,
            cache_dir,
        })
    }

    fn cache_key_for_path(&self, path: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(path.as_bytes());
        let result = hasher.finalize();
        hex_encode(result)
    }

    fn cache_paths(&self, path: &str) -> (PathBuf, PathBuf) {
        let key = self.cache_key_for_path(path);
        let body = self.cache_dir.join(format!("{}.json", key));
        let meta = self.cache_dir.join(format!("{}.meta", key));
        (body, meta)
    }

    pub fn get(&self, endpoint: &str) -> Result<Value, WanikaniError> {
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}{}", self.base_url, endpoint)
        };

        let (body_path, meta_path) = self.cache_paths(&url);

        // Prepare headers
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        let auth_val = format!("Bearer {}", self.api_key);
        headers.insert("Authorization", HeaderValue::from_str(&auth_val).map_err(|e| WanikaniError::Other(e.to_string()))?);

        // If we have cached metadata, add conditional headers
        if meta_path.exists() {
            if let Ok(mut f) = File::open(&meta_path) {
                let mut s = String::new();
                if f.read_to_string(&mut s).is_ok() {
                    for line in s.lines() {
                        if let Some(rest) = line.strip_prefix("ETag: ") {
                            headers.insert(IF_NONE_MATCH, HeaderValue::from_str(rest).unwrap_or_else(|_| HeaderValue::from_static("")));
                        }
                        if let Some(rest) = line.strip_prefix("Last-Modified: ") {
                            headers.insert(IF_MODIFIED_SINCE, HeaderValue::from_str(rest).unwrap_or_else(|_| HeaderValue::from_static("")));
                        }
                    }
                }
            }
        }

        let resp = self.client.get(&url).headers(headers).send()?;

        if resp.status().as_u16() == 304 {
            // Return cached body
            if body_path.exists() {
                let mut bf = File::open(&body_path)?;
                let mut sb = String::new();
                bf.read_to_string(&mut sb)?;
                let v: Value = serde_json::from_str(&sb)?;
                return Ok(v);
            } else {
                return Err(WanikaniError::Other("Server returned 304 but no cached body present".to_string()));
            }
        }

        let status = resp.status();
        let headers_map = resp.headers().clone();
        let text = resp.text()?;

        if status.is_success() {
            // Save body and metadata
            let mut bf = File::create(&body_path)?;
            bf.write_all(text.as_bytes())?;

            // Save relevant headers
            let mut meta_contents = String::new();
            if let Some(etag) = headers_map.get("ETag") {
                meta_contents.push_str(&format!("ETag: {}\n", etag.to_str().unwrap_or("")));
            }
            if let Some(lm) = headers_map.get("Last-Modified") {
                meta_contents.push_str(&format!("Last-Modified: {}\n", lm.to_str().unwrap_or("")));
            }
            let mut mf = File::create(&meta_path)?;
            mf.write_all(meta_contents.as_bytes())?;

            let v: Value = serde_json::from_str(&text)?;
            Ok(v)
        } else {
            Err(WanikaniError::Other(format!("HTTP {}: {}", status, text)))
        }
    }

    // Generic POST helper (no caching)
    pub fn post(&self, endpoint: &str, body: &Value) -> Result<Value, WanikaniError> {
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}{}", self.base_url, endpoint)
        };

        let auth_val = format!("Bearer {}", self.api_key);
        let resp = self.client.post(&url).header("Authorization", auth_val).json(body).send()?;
        let status = resp.status();
        let text = resp.text()?;
        if status.is_success() {
            let v: Value = serde_json::from_str(&text)?;
            Ok(v)
        } else {
            Err(WanikaniError::Other(format!("HTTP {}: {}", status, text)))
        }
    }

    pub fn put(&self, endpoint: &str, body: &Value) -> Result<Value, WanikaniError> {
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}{}", self.base_url, endpoint)
        };

        let auth_val = format!("Bearer {}", self.api_key);
        let resp = self.client.put(&url).header("Authorization", auth_val).json(body).send()?;
        let status = resp.status();
        let text = resp.text()?;
        if status.is_success() {
            let v: Value = serde_json::from_str(&text)?;
            Ok(v)
        } else {
            Err(WanikaniError::Other(format!("HTTP {}: {}", status, text)))
        }
    }

    pub fn patch(&self, endpoint: &str, body: &Value) -> Result<Value, WanikaniError> {
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}{}", self.base_url, endpoint)
        };

        let auth_val = format!("Bearer {}", self.api_key);
        let resp = self.client.patch(&url).header("Authorization", auth_val).json(body).send()?;
        let status = resp.status();
        let text = resp.text()?;
        if status.is_success() {
            let v: Value = serde_json::from_str(&text)?;
            Ok(v)
        } else {
            Err(WanikaniError::Other(format!("HTTP {}: {}", status, text)))
        }
    }

    pub fn delete(&self, endpoint: &str) -> Result<(), WanikaniError> {
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}{}", self.base_url, endpoint)
        };

        let auth_val = format!("Bearer {}", self.api_key);
        let resp = self.client.delete(&url).header("Authorization", auth_val).send()?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = resp.text().unwrap_or_default();
            Err(WanikaniError::Other(format!("HTTP {}: {}", status, text)))
        }
    }
}
