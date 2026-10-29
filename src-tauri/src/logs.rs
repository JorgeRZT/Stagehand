//! Log discovery and incremental reading.
//!
//! The UI polls `read_log` with the byte offset it last saw; we return only the new bytes.
//! Polling (instead of file watchers) keeps this identical on every OS and handles rotation
//! (file shrinks => restart from the beginning).

use crate::config::Environment;
use crate::instance::logs_dir;
use serde::Serialize;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// Max bytes returned per call, and the initial tail size when offset is None.
const CHUNK: u64 = 256 * 1024;
const INITIAL_TAIL: u64 = 64 * 1024;

#[derive(Serialize)]
pub struct LogChunk {
    pub text: String,
    /// Offset to pass on the next call.
    pub offset: u64,
}

pub fn list(env: &Environment) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(logs_dir(env))
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().is_file())
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|n| n.ends_with(".log"))
                .collect()
        })
        .unwrap_or_default();
    // Most useful first.
    names.sort_by_key(|n| match n.as_str() {
        "error.log" => (0, n.clone()),
        "stdout.log" => (1, n.clone()),
        "request.log" => (3, n.clone()),
        _ => (2, n.clone()),
    });
    names
}

pub fn read(env: &Environment, file: &str, offset: Option<u64>) -> Result<LogChunk, String> {
    // Reject path traversal: only plain file names inside the logs dir.
    if file.contains('/') || file.contains('\\') || file.contains("..") {
        return Err("Invalid log file name".into());
    }
    let path = logs_dir(env).join(file);
    let mut f = File::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let len = f.metadata().map_err(|e| e.to_string())?.len();

    let start = match offset {
        None => len.saturating_sub(INITIAL_TAIL),
        Some(o) if o > len => 0, // rotated / truncated
        Some(o) => o,
    };
    let end = len.min(start + CHUNK);
    f.seek(SeekFrom::Start(start)).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; (end - start) as usize];
    f.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(LogChunk { text: String::from_utf8_lossy(&buf).into_owned(), offset: end })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Kind;

    fn env_in(dir: &std::path::Path) -> Environment {
        Environment {
            id: "t".into(), workspace_id: "default".into(), name: "t".into(), kind: Kind::Author, dir: dir.to_string_lossy().into(),
            port: 4502, jar: String::new(), extra_runmodes: String::new(), jvm_opts: String::new(),
            start_command: String::new(), stop_command: String::new(),
        }
    }

    #[test]
    fn reads_incrementally_and_rejects_traversal() {
        let tmp = std::env::temp_dir().join(format!("aemenv-test-{}", std::process::id()));
        let logs = tmp.join("crx-quickstart/logs");
        std::fs::create_dir_all(&logs).unwrap();
        std::fs::write(logs.join("error.log"), "one\n").unwrap();
        let env = env_in(&tmp);

        let a = read(&env, "error.log", None).unwrap();
        assert_eq!(a.text, "one\n");
        std::fs::write(logs.join("error.log"), "one\ntwo\n").unwrap();
        let b = read(&env, "error.log", Some(a.offset)).unwrap();
        assert_eq!(b.text, "two\n");
        assert!(read(&env, "../x", None).is_err());
        assert_eq!(list(&env), vec!["error.log"]);
        std::fs::remove_dir_all(&tmp).ok();
    }
}
