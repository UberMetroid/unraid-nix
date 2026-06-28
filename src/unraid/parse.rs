use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

type CacheEntry = (SystemTime, HashMap<String, String>);

fn cache() -> &'static Mutex<HashMap<String, CacheEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn file_mtime(path: &str) -> Option<SystemTime> {
    std::fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

fn parse_uncached(path: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let val = line[pos + 1..].trim().trim_matches('"').to_string();
                if !val.is_empty() {
                    map.insert(key, val);
                }
            }
        }
    }
    map
}

/// Parse generic Unraid INI configuration files, caching by path + mtime.
pub fn parse_ini_file(path: &str) -> HashMap<String, String> {
    let Some(mtime) = file_mtime(path) else {
        return parse_uncached(path);
    };

    let guard = cache().lock().expect("parse_ini_file cache poisoned");
    if let Some((cached_mtime, cached_map)) = guard.get(path) {
        if *cached_mtime == mtime {
            return cached_map.clone();
        }
    }
    drop(guard);

    let map = parse_uncached(path);
    let mut guard = cache().lock().expect("parse_ini_file cache poisoned");
    guard.insert(path.to_string(), (mtime, map.clone()));
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ini_file() {
        let content = "
; This is a comment
# Another comment
key1 = value1
key2 = \"value2\"
key3=value3
";
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_ini.cfg");
        std::fs::write(&file_path, content).unwrap();

        let map = parse_ini_file(file_path.to_str().unwrap());
        assert_eq!(map.get("key1").unwrap(), "value1");
        assert_eq!(map.get("key2").unwrap(), "value2");
        assert_eq!(map.get("key3").unwrap(), "value3");

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_parse_ini_file_empty_values_omitted() {
        // Empty values should be skipped so that consumers' default values apply.
        let content = "
present = hello
empty =
quoted_empty = \"\"
";
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "test_ini_empty-{}.cfg",
            std::process::id()
        ));
        std::fs::write(&file_path, content).unwrap();

        let map = parse_ini_file(file_path.to_str().unwrap());
        assert_eq!(map.get("present").unwrap(), "hello");
        assert!(map.get("empty").is_none(), "empty value should be skipped");
        assert!(map.get("quoted_empty").is_none(), "quoted empty value should be skipped");

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_parse_ini_file_cache_hit_within_mtime() {
        let content_v1 = "k1 = v1\nk2 = v2\n";
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "test_ini_cache_hit-{}.cfg",
            std::process::id()
        ));
        std::fs::write(&file_path, content_v1).unwrap();

        let map_a = parse_ini_file(file_path.to_str().unwrap());
        assert_eq!(map_a.get("k1").unwrap(), "v1");

        let map_b = parse_ini_file(file_path.to_str().unwrap());
        assert_eq!(map_a, map_b, "cached map should match prior parse");

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    #[allow(clippy::duration_suboptimal_units)]
    fn test_parse_ini_file_reparses_after_mtime_change() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "test_ini_cache_mtime-{}.cfg",
            std::process::id()
        ));
        std::fs::write(&file_path, "k1 = v1\n").unwrap();

        let map_a = parse_ini_file(file_path.to_str().unwrap());
        assert_eq!(map_a.get("k1").unwrap(), "v1");

        std::fs::write(&file_path, "k1 = v2\nk2 = new\n").unwrap();
        let new_mtime = std::fs::File::options()
            .write(true)
            .open(&file_path)
            .and_then(|f| {
                let target = std::time::SystemTime::now()
                    + std::time::Duration::from_secs(60);
                f.set_modified(target)?;
                Ok(target)
            })
            .expect("set_modified should succeed");
        let actual_mtime = std::fs::metadata(&file_path)
            .and_then(|m| m.modified())
            .expect("metadata should be readable");
        assert_eq!(actual_mtime, new_mtime, "mtime must be moved forward for this test");

        let map_b = parse_ini_file(file_path.to_str().unwrap());
        assert_eq!(
            map_b.get("k1").unwrap(),
            "v2",
            "mtime change must invalidate cache"
        );
        assert_eq!(map_b.get("k2").unwrap(), "new");

        let _ = std::fs::remove_file(&file_path);
    }
}