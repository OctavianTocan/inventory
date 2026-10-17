//! Node's `path` functions and file system errors as the JS server saw them.

use std::path::{Component, Path, PathBuf};

/// `path.resolve(base, path)` on POSIX: lexical, `..` stopping at the file system root, no trailing slash.
///
/// # Arguments
/// * `base` - Absolute directory.
/// * `path` - Path relative to `base`, or absolute.
pub fn resolve(base: &Path, path: &str) -> PathBuf {
    let joined = if path.starts_with('/') { PathBuf::from(path) } else { base.join(path) };
    let mut out = PathBuf::from("/");
    for part in joined.components() {
        match part {
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(name) => out.push(name),
            _ => {}
        }
    }
    out
}

/// `path.join(base, path)` on POSIX for an absolute `base`: both joined by `/` with `.` and `..` resolved lexically,
/// `..` stopping at the root, and a trailing `/` of `path` kept.
///
/// # Arguments
/// * `base` - Absolute directory.
/// * `path` - Path below `base`; a leading `/` does not make it absolute.
pub fn join(base: &Path, path: &str) -> PathBuf {
    let base = base.to_string_lossy();
    let joined = if path.is_empty() { base.into_owned() } else { format!("{base}/{path}") };
    let mut parts: Vec<&str> = Vec::new();
    for part in joined.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    let mut out = format!("/{}", parts.join("/"));
    if joined.ends_with('/') && !parts.is_empty() {
        out.push('/');
    }
    PathBuf::from(out)
}

/// `path.extname(path)` on POSIX: the last `.` and what follows it in the last segment, or empty for a segment
/// without one, one that only starts with a dot, or `..`.
///
/// # Arguments
/// * `path` - A path.
pub fn extname(path: &str) -> &str {
    let bytes = path.as_bytes();
    let (mut start_dot, mut start_part, mut end) = (None, 0, None);
    let mut matched_slash = true;
    let mut pre_dot_state = 0;
    for index in (0..bytes.len()).rev() {
        let byte = bytes[index];
        if byte == b'/' {
            if !matched_slash {
                start_part = index + 1;
                break;
            }
            continue;
        }
        if end.is_none() {
            matched_slash = false;
            end = Some(index + 1);
        }
        if byte == b'.' {
            if start_dot.is_none() {
                start_dot = Some(index);
            } else if pre_dot_state != 1 {
                pre_dot_state = 1;
            }
        } else if start_dot.is_some() {
            pre_dot_state = -1;
        }
    }
    let (Some(start_dot), Some(end)) = (start_dot, end) else { return "" };
    if pre_dot_state == 0 || (pre_dot_state == 1 && start_dot == end - 1 && start_dot == start_part + 1) {
        return "";
    }
    &path[start_dot..end]
}

// libuv's error name and description for an I/O error, as Node prints them.
fn describe(error: &std::io::Error) -> (String, &'static str) {
    let known = [
        (2, "ENOENT", "no such file or directory"),
        (13, "EACCES", "permission denied"),
        (21, "EISDIR", "illegal operation on a directory"),
        (20, "ENOTDIR", "not a directory"),
        (17, "EEXIST", "file already exists"),
        (1, "EPERM", "operation not permitted"),
        (40, "ELOOP", "too many symbolic links encountered"),
        (36, "ENAMETOOLONG", "name too long"),
        (24, "EMFILE", "too many open files"),
        (28, "ENOSPC", "no space left on device"),
        (30, "EROFS", "read-only file system"),
        (5, "EIO", "i/o error"),
        (98, "EADDRINUSE", "address already in use"),
        (99, "EADDRNOTAVAIL", "address not available"),
    ];
    match error.raw_os_error().and_then(|code| known.iter().find(|(number, ..)| *number == code)) {
        Some((_, name, description)) => ((*name).to_owned(), description),
        None => (format!("Unknown system error {}", -error.raw_os_error().unwrap_or(0)), "unknown error"),
    }
}

/// `String(error)` of the `TypeError` a `fs` call throws for a path holding a NUL byte (`ERR_INVALID_ARG_VALUE`).
///
/// # Arguments
/// * `path` - The path the call was given; the message shows it inspected, cut at 128 characters.
pub fn null_bytes_error(path: &Path) -> String {
    let inspected = mm_js::inspect::inspect(&mm_js::Value::from(path.to_string_lossy().as_ref()));
    let inspected = if mm_js::text::len(&inspected) > 128 { format!("{}...", mm_js::text::slice(&inspected, 0, Some(128))) } else { inspected };
    format!("TypeError [ERR_INVALID_ARG_VALUE]: The argument 'path' must be a string, Uint8Array, or URL without null bytes. Received {inspected}")
}

/// `String(error)` of the error a `fs` promise rejected with: `Error: CODE: description, syscall 'path'`.
///
/// # Arguments
/// * `error` - The I/O error.
/// * `syscall` - The failing call, such as `open` or `scandir`.
/// * `path` - The path Node names in the message, if any.
pub fn fs_error(error: &std::io::Error, syscall: &str, path: Option<&Path>) -> String {
    if let Some(path) = path.filter(|path| error.kind() == std::io::ErrorKind::InvalidInput && path.as_os_str().as_encoded_bytes().contains(&0)) {
        return null_bytes_error(path);
    }
    let (name, description) = describe(error);
    match path {
        Some(path) => format!("Error: {name}: {description}, {syscall} '{}'", path.display()),
        None => format!("Error: {name}: {description}, {syscall}"),
    }
}

/// `String(error)` of the error `server.listen` emits when it cannot bind.
///
/// # Arguments
/// * `error` - The I/O error.
/// * `host` - The address it listened on.
/// * `port` - The port it listened on.
pub fn listen_error(error: &std::io::Error, host: &str, port: u16) -> String {
    let (name, description) = describe(error);
    format!("Error: listen {name}: {description} {host}:{port}")
}

/// `readFile(path)`: the file's bytes, or `String(error)` of what Node would have thrown (a directory fails at `read`,
/// anything else at `open`).
///
/// # Arguments
/// * `path` - The file.
///
/// # Errors
/// The JS error text.
pub async fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    tokio::fs::read(path).await.map_err(|error| {
        if error.kind() == std::io::ErrorKind::IsADirectory { fs_error(&error, "read", None) } else { fs_error(&error, "open", Some(path)) }
    })
}
