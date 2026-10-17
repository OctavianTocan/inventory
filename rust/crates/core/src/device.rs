//! Port of `lib/device.mjs`.

/// The device class mymind's server renders differently for (`deviceVariant`).
///
/// # Arguments
/// * `user_agent` - Request user agent; empty when the request has none.
///
/// # Returns
/// `iphone`, `ipad`, `android-phone`, `android-tablet`, or empty for desktop.
pub fn device_variant(user_agent: &str) -> &'static str {
    if user_agent.contains("iPhone") || user_agent.contains("iPod") {
        return "iphone";
    }
    if user_agent.contains("iPad") {
        return "ipad";
    }
    if user_agent.contains("Android") {
        // `/Android.*Mobile/`: `.` stops at line terminators, so Mobile must follow on the same line.
        let mobile = user_agent.match_indices("Android").any(|(start, _)| {
            let line = &user_agent[start..];
            let end = line.find(['\n', '\r', '\u{2028}', '\u{2029}']).unwrap_or(line.len());
            line[..end].contains("Mobile")
        });
        return if mobile { "android-phone" } else { "android-tablet" };
    }
    ""
}
