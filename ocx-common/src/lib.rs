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
