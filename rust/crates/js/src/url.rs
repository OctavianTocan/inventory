//! `new URL(input, base)` as Node parses it, on top of the `url` crate.

pub use url::Url;
use url::Position;

/// `new URL(input)` or `new URL(input, base)`, or `None` where the constructor throws.
///
/// # Arguments
/// * `input` - URL text, absolute or relative to `base`.
/// * `base` - Base URL text.
pub fn parse(input: &str, base: Option<&str>) -> Option<Url> {
    let url = match base {
        None => Url::parse(input).ok()?,
        Some(base) => {
            let base = Url::parse(base).ok()?;
            let input = if names_host(base.scheme()) { authority_slashes(input) } else { input.into() };
            Url::options().base_url(Some(&base)).parse(&input).ok()?
        }
    };
    // Two changes to the URL Standard after the `url` crate's release that Node follows: `^` joined the path
    // percent-encode set, and an opaque path's last space before a query or fragment is encoded.
    let path = if url.cannot_be_a_base() {
        match url.path().strip_suffix(' ') {
            Some(rest) if url.query().is_some() || url.fragment().is_some() => format!("{rest}%20"),
            _ => return Some(url),
        }
    } else if url.path().contains('^') {
        url.path().replace('^', "%5E")
    } else {
        return Some(url);
    };
    Url::parse(&format!("{}{path}{}", &url[..Position::BeforePath], &url[Position::AfterPath..])).ok()
}

// Special schemes whose scheme-relative inputs name a host (`file:` has a state of its own).
fn names_host(scheme: &str) -> bool {
    matches!(scheme, "http" | "https" | "ws" | "wss" | "ftp")
}

// A scheme-relative input with its run of leading slashes and backslashes cut to `//`: on such a base the URL
// Standard skips them all before the host, where the `url` crate rejects some.
fn authority_slashes(input: &str) -> std::borrow::Cow<'_, str> {
    let trimmed = input.trim_start_matches(|c: char| c <= ' ');
    let mut slashes = 0;
    for (index, c) in trimmed.char_indices() {
        match c {
            '/' | '\\' => slashes += 1,
            '\t' | '\n' | '\r' => {}
            _ if slashes >= 2 => return format!("//{}", &trimmed[index..]).into(),
            _ => break,
        }
    }
    input.into()
}

/// `url.search`: the query with its `?`, or empty when it is missing or empty.
///
/// # Arguments
/// * `url` - Parsed URL.
pub fn search(url: &Url) -> String {
    match url.query() {
        Some(query) if !query.is_empty() => format!("?{query}"),
        _ => String::new(),
    }
}

/// `url.hash`: the fragment with its `#`, or empty when it is missing or empty.
///
/// # Arguments
/// * `url` - Parsed URL.
pub fn hash(url: &Url) -> String {
    match url.fragment() {
        Some(fragment) if !fragment.is_empty() => format!("#{fragment}"),
        _ => String::new(),
    }
}

/// `url.host`: host and non-default port, or empty.
///
/// # Arguments
/// * `url` - Parsed URL.
pub fn host(url: &Url) -> String {
    let hostname = url.host_str().unwrap_or("");
    url.port().map_or_else(|| hostname.to_owned(), |port| format!("{hostname}:{port}"))
}

/// `url.origin`: `scheme://host[:port]` for special schemes, `null` for others.
///
/// # Arguments
/// * `url` - Parsed URL.
pub fn origin(url: &Url) -> String {
    url.origin().ascii_serialization()
}
