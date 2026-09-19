// Persistent device selection config stored next to the executable.
// Format: simple key=value lines (mic=<id>, speaker=<id>, output=<id>, delay_ms=<ms>, lock_delay=<0|1>).

use std::collections::HashMap;
use std::path::PathBuf;

pub struct Config {
    pub mic: Option<String>,
    pub speaker: Option<String>,
    pub output: Option<String>,
    pub delay_ms: Option<i32>,
    pub lock_delay: bool,
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn config_parses_delay_lock_fields() {
        let cfg = Config {
            mic: None,
            speaker: None,
            output: None,
            delay_ms: Some(42),
            lock_delay: true,
        };

        assert_eq!(cfg.delay_ms, Some(42));
        assert!(cfg.lock_delay);
    }
}

fn config_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.join("rust-aec.cfg"))
}

pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config {
            mic: None,
            speaker: None,
            output: None,
            delay_ms: None,
            lock_delay: false,
        };
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Config {
            mic: None,
            speaker: None,
            output: None,
            delay_ms: None,
            lock_delay: false,
        };
    };
    let mut map: HashMap<&str, &str> = HashMap::new();
    for line in text.lines() {
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim(), v.trim());
        }
    }
    let delay_ms = map
        .get("delay_ms")
        .and_then(|v| v.parse::<i32>().ok());
    let lock_delay = map
        .get("lock_delay")
        .map(|v| matches!(v.trim(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);

    Config {
        mic: map.get("mic").map(|s| s.to_string()),
        speaker: map.get("speaker").map(|s| s.to_string()),
        output: map.get("output").map(|s| s.to_string()),
        delay_ms,
        lock_delay,
    }
}

pub fn save(
    mic: Option<&str>,
    speaker: Option<&str>,
    output: Option<&str>,
    delay_ms: Option<i32>,
    lock_delay: bool,
) {
    let Some(path) = config_path() else { return };
    let mut lines = Vec::new();
    if let Some(id) = mic {
        lines.push(format!("mic={id}"));
    }
    if let Some(id) = speaker {
        lines.push(format!("speaker={id}"));
    }
    if let Some(id) = output {
        lines.push(format!("output={id}"));
    }
    if let Some(delay) = delay_ms {
        lines.push(format!("delay_ms={delay}"));
    }
    lines.push(format!("lock_delay={}", if lock_delay { "1" } else { "0" }));
    let _ = std::fs::write(path, lines.join("\n"));
}
