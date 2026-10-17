//! Port of `lib/render/media.mjs`: the `mymind-image` element and its rendition URLs.

use mm_js::number::{number_to_string, round};
use mm_js::{Value, obj};

use super::access::{member, min, text};
use super::encoding::escape_html as e;
use crate::Result;

/// Output pixel dimensions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    /// Width in pixels.
    pub width: f64,
    /// Height in pixels.
    pub height: f64,
}

/// The common divisor the server's CSS aspect ratios use (`gcd`).
pub(crate) fn gcd(mut a: f64, mut b: f64) -> f64 {
    while b != 0.0 && !b.is_nan() {
        (a, b) = (b, a % b);
    }
    a
}

/// Fit intrinsic media dimensions into a bounding box without upscaling (`fit`).
///
/// # Arguments
/// * `media` - Object with the intrinsic `width` and `height` in pixels.
/// * `width` - Maximum output width.
/// * `height` - Maximum output height.
/// * `even` - Round to the nearest even pixel, as video needs, instead of flooring.
///
/// # Errors
/// `media` is `null` or `undefined`.
pub fn fit(media: &Value, width: f64, height: f64, even: bool) -> Result<Size> {
    Ok(fit_dimensions(member(media, "width")?.to_number(), member(media, "height")?.to_number(), width, height, even))
}

/// `fit` for plain numbers.
pub fn fit_dimensions(media_width: f64, media_height: f64, width: f64, height: f64, even: bool) -> Size {
    let scale = min(min(1.0, width / media_width), height / media_height);
    let round_to = |n: f64| if even { round(n / 2.0) * 2.0 } else { (n + 1e-9).floor() };
    Size { width: round_to(media_width * scale), height: round_to(media_height * scale) }
}

/// Render a prepared cover (`renderMedia`).
///
/// # Arguments
/// * `media` - Cover `{src, width, height, dominantColor?, type?, svg?, imageWidth?, imageHeight?}`; a falsy value
///   renders nothing.
///
/// # Returns
/// A `mymind-image` element without a trailing newline.
pub fn render_media(media: &Value) -> String {
    if !media.truthy() {
        return String::new();
    }
    let width = media.get("width").to_number();
    let height = media.get("height").to_number();
    let divisor = gcd(width, height);
    let color = media.get("dominantColor");
    let style = format!("aspect-ratio:{}/{}{}", number_to_string(width / divisor), number_to_string(height / divisor),
        if color.truthy() { format!(";background-color:{}", color.to_js_string()) } else { String::new() });
    let source = e(&text(media.get("src")));
    let svg = media.get("svg").truthy();
    let image = if media.get("type").as_str() == Some("video") {
        format!("<video autoplay muted playsinline loop  class=\"lazy\" data-src=\"{source}\"></video>")
    } else {
        let size = if svg {
            let pick = |own: &Value, fallback: &Value| own.or(fallback).to_js_string();
            format!(" width=\"{}\" height=\"{}\"", pick(media.get("imageWidth"), media.get("width")), pick(media.get("imageHeight"), media.get("height")))
        } else {
            String::new()
        };
        format!("<img class=\"lazy\" data-src=\"{source}\"{size}/>")
    };
    format!("<mymind-image style=\"{}\"{}>{image}</mymind-image>", e(&style), if svg { " class=\"svg\"" } else { "" })
}

/// Derive the image cover of an uploaded blob (`imageCover`).
///
/// # Arguments
/// * `blob` - Uploaded image metadata `{path, width, height, type, dominantColor?}`.
///
/// # Returns
/// A cover object that `render_media` accepts.
///
/// # Errors
/// `blob` is `null` or `undefined`.
pub fn image_cover(blob: &Value) -> Result<Value> {
    let size = fit(blob, 800.0, 1000.0, false)?;
    let gif = blob.get("type").as_str() == Some("image/gif");
    Ok(obj! {
        "width" => size.width,
        "height" => size.height,
        "dominantColor" => blob.get("dominantColor"),
        "type" => if gif { "video" } else { "image" },
        "src" => format!("/media/{};{}x{}.{}", blob.get("path").to_js_string(), number_to_string(size.width), number_to_string(size.height), if gif { "mp4" } else { "webp" }),
    }
    .into())
}

/// Render the poster and hover preview of an uploaded video (`renderVideo`).
///
/// # Arguments
/// * `blob` - Uploaded video metadata `{path, width, height}`.
///
/// # Returns
/// A `mymind-image` element holding both renditions.
///
/// # Errors
/// `blob` is `null` or `undefined`.
pub fn render_video(blob: &Value) -> Result<String> {
    let poster = fit(blob, 800.0, 1000.0, true)?;
    let preview = fit(blob, 1280.0, 1280.0, true)?;
    let divisor = gcd(poster.width, poster.height);
    let path = blob.get("path").to_js_string();
    let still = format!("/media/{path};{}x{}.jpeg", number_to_string(poster.width), number_to_string(poster.height));
    let clip = format!("/media/{path};{}x{}/hevc.mp4", number_to_string(preview.width), number_to_string(preview.height));
    Ok(format!(
        "<mymind-image style=\"aspect-ratio:{}/{}\"><img class=\"lazy\" data-src=\"{}\" /><video class=\"hover-preview\" playsinline muted loop src=\"{}\"></video></mymind-image>",
        number_to_string(poster.width / divisor), number_to_string(poster.height / divisor), e(&still), e(&clip)
    ))
}
