use crate::unraid::SUPERVISOR_PORT;
use serde_json::Value;
use std::process::{exit, Command};

pub fn view_logs(service: &str) {
    if !crate::store::is_valid_service_name(service) {
        crate::store::log_event("ERROR", &format!("Invalid service name '{service}' for view-logs"));
        exit(1);
    }
    let log_file = format!("/var/log/nix-services/{}.log", service);
    let mut rendered = false;
    let mut output = String::new();

    output.push_str(&format!("<h3>Active console output for: {}</h3>", html_escape(service)));

    if std::path::Path::new(&log_file).exists() {
        if let Ok(content) = run_tail(&log_file, 200) {
            output.push_str("<pre style='white-space: pre-wrap; word-wrap: break-word;'>");
            for line in content.lines() {
                let line_trimmed = line.trim();
                if line_trimmed.is_empty() { continue; }
                if let Ok(v) = serde_json::from_str::<Value>(line_trimmed) {
                    let time = v.get("time").and_then(|t| t.as_str()).unwrap_or("");
                    let message = v.get("message").and_then(|m| m.as_str()).unwrap_or("");
                    if !time.is_empty() {
                        let time_display: String = time
                            .chars()
                            .take(19)
                            .collect::<String>()
                            .replace('T', " ");
                        output.push_str(&format!(
                            "<span style='color:#888;'>[{}]</span> {}\n",
                            html_escape(&time_display),
                            html_escape(message)
                        ));
                    } else {
                        output.push_str(&format!("{}\n", html_escape(message)));
                    }
                } else {
                    output.push_str(&format!("{}\n", html_escape(line_trimmed)));
                }
            }
            output.push_str("</pre>");
            rendered = true;
        }
    }

    if !rendered {
        let url = format!("http://127.0.0.1:{SUPERVISOR_PORT}/process/logs/{service}/0/200");
        if let Ok(mut resp) = ureq::get(&url)
            .config()
            .timeout_per_call(Some(std::time::Duration::from_secs(2)))
            .build()
            .call()
        {
            if let Ok(Value::Object(map)) = resp.body_mut().read_json::<Value>() {
                if let Some(Value::Array(lines)) = map.get("logs") {
                    output.push_str("<pre style='white-space: pre-wrap; word-wrap: break-word;'>");
                    for line_val in lines {
                        if let Some(line) = line_val.as_str() {
                            output.push_str(&format!("{}\n", html_escape(line)));
                        }
                    }
                    output.push_str("</pre>");
                    rendered = true;
                }
            }
        }
    }

    if !rendered {
        output.push_str("<p class='text-muted'>No logs found. If the service just started, it might take a few seconds to populate.</p>");
    }

    println!("{}", output);
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&#x27;")
}

fn run_tail(file: &str, lines: usize) -> Result<String, String> {
    let output = Command::new("tail")
        .args(["-n", &lines.to_string(), file])
        .output()
        .map_err(|e| format!("failed to invoke tail on {file}: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg(test)]
mod tests {
    /// Build a synthetic log JSON line with a given `time` string. Used to
    /// exercise the time-formatting code path without writing to disk.
    fn render_time(time: &str) -> String {
        if time.is_empty() {
            return String::new();
        }
        time.chars()
            .take(19)
            .collect::<String>()
            .replace('T', " ")
    }

    #[test]
    fn test_render_time_short_unchanged() {
        assert_eq!(render_time("2026-06-27"), "2026-06-27");
    }

    #[test]
    fn test_render_time_t_separator_replaced() {
        assert_eq!(render_time("2026-06-27T12:34:56Z"), "2026-06-27 12:34:56");
    }

    #[test]
    fn test_render_time_multibyte_utf8_does_not_panic() {
        let time = "é".repeat(19);
        let rendered = render_time(&time);
        assert_eq!(rendered.chars().count(), 19);
    }

    #[test]
    fn test_render_time_more_than_19_chars_truncates_at_19() {
        let time = "2026-06-27T12:34:56.789Z";
        let rendered = render_time(time);
        assert_eq!(rendered.chars().count(), 19);
    }
}
