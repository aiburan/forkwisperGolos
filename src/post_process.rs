use serde_json::json;
use std::time::Duration;

pub const DEFAULT_SYSTEM_PROMPT: &str = "You are an editor of spoken thoughts. Convert the raw voice transcript into clear written text.
Reply in the SAME language as the input. Preserve meaning exactly; do NOT add facts.
Remove filler, false starts, repetitions; fix word order and obvious errors.
Keep all names, numbers and technical terms. If style is '{style}' and it's a task/list,
organize into clear points; otherwise one clear paragraph. Output ONLY the final text.";

/// Send a raw transcript to an OpenAI-compatible chat endpoint and return cleaned text.
pub async fn process(
    base_url: &str,
    api_key: &str,
    model: &str,
    style: &str,
    prompt_path: &std::path::Path,
    raw: &str,
) -> Result<String, String> {
    // Validate URL scheme — reject file://, ftp://, etc.
    if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
        return Err("Invalid API URL: only http:// and https:// are allowed".into());
    }

    // Warn (in stderr) when using unencrypted HTTP for non-localhost
    if base_url.starts_with("http://")
        && !base_url.starts_with("http://localhost")
        && !base_url.starts_with("http://127.0.0.1")
        && !base_url.starts_with("http://[::1]")
    {
        eprintln!("WARNING: post-processing endpoint uses unencrypted HTTP for a remote host");
    }

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let template = std::fs::read_to_string(prompt_path)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SYSTEM_PROMPT.to_string());
    let system_prompt = template.replace("{style}", style);
    let body = json!({
        "model": model,
        "temperature": 0.2,
        "messages": [
            {
                "role": "system",
                "content": system_prompt,
            },
            {
                "role": "user",
                "content": raw,
            },
        ],
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let resp = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("API error {status}: {body}"));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;

    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("No choices[0].message.content field in response: {json}"))
}
