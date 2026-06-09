use std::path::PathBuf;

pub fn read_opencode_config(local: bool) -> String {
    if local {
        "local config (stub)".to_string()
    } else {
        "global config (stub)".to_string()
    }
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
