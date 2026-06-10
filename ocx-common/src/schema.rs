use std::io::Read;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

use jsonschema::{Retrieve, Uri};

static SCHEMA_INIT: OnceLock<Result<(), String>> = OnceLock::new();

pub fn embedded_config_schema() -> &'static [u8] {
    include_bytes!(concat!(env!("OUT_DIR"), "/config.json"))
}

pub fn embedded_etag() -> &'static str {
    include_str!(concat!(env!("OUT_DIR"), "/etag.txt"))
}

fn schema_cache_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    base.join("ocx")
}

fn cached_config_path() -> PathBuf {
    schema_cache_dir().join("config.json")
}

fn cached_etag_path() -> PathBuf {
    schema_cache_dir().join("etag.txt")
}

fn should_check_for_update() -> bool {
    std::fs::metadata(cached_etag_path())
        .and_then(|m| m.modified())
        .map(|t| t.elapsed().map(|d| d > Duration::from_secs(3600)).unwrap_or(true))
        .unwrap_or(true)
}

fn atomic_write(path: &std::path::Path, data: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, data).map_err(|e| format!("write failed: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("rename failed: {e}"))?;
    Ok(())
}

fn init_schema() -> Result<(), String> {
    SCHEMA_INIT
        .get_or_init(|| {
            let dir = schema_cache_dir();
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("failed to create {:?}: {e}", dir))?;

            let config_path = cached_config_path();
            let etag_path = cached_etag_path();

            if !config_path.exists() {
                atomic_write(&config_path, embedded_config_schema())?;
                atomic_write(&etag_path, embedded_etag().as_bytes())?;
            }

            if should_check_for_update() {
                if let Err(e) = try_update() {
                    eprintln!("warning: schema update check failed: {e}");
                }
            }

            Ok(())
        })
        .clone()
}

fn try_update() -> Result<(), String> {
    let current_etag = std::fs::read_to_string(cached_etag_path()).unwrap_or_default();
    let url = "https://opencode.ai/config.json";

    let request = ureq::get(url).header("If-None-Match", &format!("W/\"{}\"", current_etag));

    let response = match request.call() {
        Ok(r) => r,
        Err(e) => return Err(format!("request failed: {e}")),
    };

    match response.status().as_u16() {
        200 => {
            let new_etag = response
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(|v| {
                    v.trim()
                        .trim_start_matches("W/")
                        .trim_matches('"')
                        .to_string()
                })
                .unwrap_or_default();

            let mut body = Vec::new();
            response
                .into_body()
                .into_reader()
                .read_to_end(&mut body)
                .map_err(|e| format!("failed to read response body: {e}"))?;

            atomic_write(&cached_config_path(), &body)?;
            atomic_write(&cached_etag_path(), new_etag.as_bytes())?;

            Ok(())
        }
        304 => {
            std::fs::write(cached_etag_path(), current_etag.as_bytes())
                .map_err(|e| format!("failed to touch etag file: {e}"))
        }
        status => {
            let _ = response.into_body().into_reader().read_to_end(&mut Vec::new());
            Err(format!("unexpected status code {status}"))
        }
    }
}

static COMPILED_SCHEMA: OnceLock<Result<jsonschema::Validator, String>> = OnceLock::new();

fn get_validator() -> Result<&'static jsonschema::Validator, String> {
    init_schema()?;

    COMPILED_SCHEMA
        .get_or_init(|| {
            let schema_bytes = std::fs::read(cached_config_path())
                .map_err(|e| format!("failed to read cached schema: {e}"))?;
            let schema: serde_json::Value = serde_json::from_slice(&schema_bytes)
                .map_err(|e| format!("invalid schema JSON: {e}"))?;
            jsonschema::validator_for(&schema)
                .map_err(|e| format!("schema compilation failed: {e}"))
        })
        .as_ref()
        .map_err(|e| e.clone())
}

pub fn validate_config(value: &serde_json::Value) -> Result<(), Vec<String>> {
    let validator = get_validator().map_err(|e| vec![e])?;
    let errors: Vec<String> = validator
        .iter_errors(value)
        .map(|e| e.to_string())
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

struct NoResolveRetriever;

impl Retrieve for NoResolveRetriever {
    fn retrieve(
        &self,
        _uri: &Uri<String>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::json!({}))
    }
}

static COMPILED_AGENT_SCHEMA: OnceLock<Result<jsonschema::Validator, String>> = OnceLock::new();

fn get_agent_validator() -> Result<&'static jsonschema::Validator, String> {
    init_schema()?;

    COMPILED_AGENT_SCHEMA
        .get_or_init(|| {
            let schema_bytes = std::fs::read(cached_config_path())
                .map_err(|e| format!("failed to read cached schema: {e}"))?;
            let full_schema: serde_json::Value = serde_json::from_slice(&schema_bytes)
                .map_err(|e| format!("invalid schema JSON: {e}"))?;

            let mut defs = full_schema["$defs"].clone();
            if let Some(model) = defs
                .pointer_mut("/AgentConfig/properties/model")
                .and_then(|v| v.as_object_mut())
            {
                model.remove("$ref");
            }

            let agent_schema = serde_json::json!({
                "$ref": "#/$defs/AgentConfig",
                "$defs": defs
            });

            jsonschema::options()
                .with_retriever(NoResolveRetriever)
                .build(&agent_schema)
                .map_err(|e| format!("agent schema compilation failed: {e}"))
        })
        .as_ref()
        .map_err(|e| e.clone())
}

pub fn validate_agent_config(value: &serde_json::Value) -> Result<(), Vec<String>> {
    let validator = get_agent_validator().map_err(|e| vec![e])?;
    let errors: Vec<String> = validator
        .iter_errors(value)
        .map(|e| e.to_string())
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_schema_is_valid_json() {
        let schema = embedded_config_schema();
        let result: Result<serde_json::Value, _> = serde_json::from_slice(schema);
        assert!(result.is_ok(), "embedded schema is not valid JSON");
    }

    #[test]
    fn embedded_etag_is_not_empty() {
        let etag = embedded_etag();
        assert!(!etag.is_empty(), "embedded etag should not be empty");
    }
}
