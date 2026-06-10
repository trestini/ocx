use std::io::Read;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let (body, raw_etag) = fetch_config("https://opencode.ai/config.json")
        .or_else(|_| fetch_config("http://opencode.ai/config.json"))
        .expect("failed to download config.json from both https:// and http://opencode.ai/config.json");

    let etag = raw_etag
        .unwrap_or_default()
        .trim()
        .trim_start_matches("W/")
        .trim_matches('"')
        .to_string();

    std::fs::write(format!("{out_dir}/config.json"), &body).unwrap();
    std::fs::write(format!("{out_dir}/etag.txt"), etag.as_bytes()).unwrap();
}

fn fetch_config(url: &str) -> Result<(Vec<u8>, Option<String>), String> {
    let response = ureq::get(url).call().map_err(|e| format!("{url}: {e}"))?;

    let etag = response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());

    let mut body = Vec::new();
    response
        .into_body()
        .into_reader()
        .read_to_end(&mut body)
        .map_err(|e| format!("{url}: failed to read body: {e}"))?;

    Ok((body, etag))
}
