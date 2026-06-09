use std::fs;
use std::path::{Path, PathBuf};

pub fn read_opencode_config(local: bool) -> Result<serde_json::Value, String> {
    let path = if local {
        Path::new(".opencode/opencode.json")
    } else {
        // TODO: resolve global config path
        return Err("global config not implemented yet".to_string());
    };

    let content = fs::read_to_string(path).map_err(|e| format!("failed to read config: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("invalid JSON in config: {e}"))
}

pub fn write_local_config(value: &serde_json::Value) -> Result<(), String> {
    let path = Path::new(".opencode/opencode.json");
    let content =
        serde_json::to_string_pretty(value).map_err(|e| format!("failed to serialize: {e}"))?;
    fs::write(path, content).map_err(|e| format!("failed to write config: {e}"))
}

pub fn read_agent_markdown(path: &str) -> (serde_yaml::Value, String) {
    let header: serde_yaml::Value = serde_yaml::from_str("name: stub-agent").unwrap();
    let body = format!("agent markdown body from `{path}` (stub)");
    (header, body)
}

pub fn list_system_agents() -> Vec<(String, PathBuf)> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };

    let dirs = vec![
        home.join(".config/opencode/agents"),
        home.join(".agents/agents"),
        home.join(".config/agents/agents"),
    ];

    let mut entries: Vec<(String, PathBuf)> = Vec::new();

    for dir in &dirs {
        if !dir.is_dir() {
            continue;
        }
        let read = match std::fs::read_dir(dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    entries.push((stem.to_string(), path));
                }
            }
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    entries
}

pub fn read_and_convert_agent(path: &Path) -> Result<(String, serde_json::Value), String> {
    let content = fs::read_to_string(path).map_err(|e| format!("failed to read file: {e}"))?;

    let rest = content
        .strip_prefix("---\n")
        .ok_or_else(|| "missing opening `---`".to_string())?;

    let (yaml_str, body) = rest
        .split_once("\n---\n")
        .ok_or_else(|| "missing closing `---`".to_string())?;

    let body = body.trim();

    // Bare * keys (which conflict with YAML alias syntax) are auto-quoted to "*"
    let yaml_str = yaml_str
        .lines()
        .map(|line| {
            if let Some(rest) = line.trim_start().strip_prefix("*:") {
                let indent = &line[..line.len() - rest.len() - 2];
                format!("{indent}\"*\":{}", rest)
            } else if let Some(rest) = line.trim_start().strip_prefix("* ") {
                let indent = &line[..line.len() - rest.len() - 2];
                format!("{indent}\"*\" {}", rest)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut base: serde_json::Value =
        serde_yaml::from_str(&yaml_str).map_err(|e| format!("invalid YAML: {e}"))?;

    let agent_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "invalid filename".to_string())?
        .to_string();

    if let Some(obj) = base.as_object_mut() {
        obj.insert(
            "prompt".to_string(),
            serde_json::Value::String(body.to_string()),
        );
    }

    Ok((agent_name, base))
}

pub fn add_agent_to_config(name: &str) -> Result<serde_json::Value, String> {
    let system_agents = list_system_agents();

    let agent_path = system_agents
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, p)| p.clone())
        .ok_or_else(|| format!("Agent {name} doesn't exists"))?;

    let (_agent_name, agent_value) = read_and_convert_agent(&agent_path)?;

    let mut config = read_opencode_config(true)?;

    if let Some(obj) = config.as_object_mut() {
        if !obj.contains_key("agent") {
            obj.insert("agent".to_string(), serde_json::json!({}));
        }
        if let Some(agent_obj) = obj.get_mut("agent").and_then(|v| v.as_object_mut()) {
            agent_obj.insert(name.to_string(), agent_value);
        }
    }

    Ok(config)
}
