//! ThumbHash previews (<https://evanw.github.io/thumbhash/>): a card's gallery picture blurred into about 25 bytes,
//! which the app draws the moment it shows a tile while the real picture loads, as the image server of Omar's Wholana
//! does.

use std::io::Cursor;

use image_webp::WebPDecoder;
use mm_js::{Value, base64, json};

use crate::host::Media;
use crate::media_files::cover;
use crate::store::{GRID_HIDDEN_FLAGS, Store};
use crate::store_write::Writer;
use crate::{Error, Result};

// The encoder reads at most this many pixels on each side.
const SIDE: usize = 100;
// Largest picture decoded, the media grammar's largest rendition.
const MAX_PIXELS: u64 = 4096 * 4096;

fn unreadable() -> Error {
    Error::status(415, "Picture is not a readable WebP")
}

/// The ThumbHash of a WebP picture.
///
/// # Arguments
/// * `bytes` - The WebP file; an animated one gives its first frame.
///
/// # Returns
/// The hash in standard base64.
///
/// # Errors
/// Status 415 for bytes that are not a WebP of at most 4096 by 4096 pixels.
pub fn of_webp(bytes: &[u8]) -> Result<String> {
    let mut decoder = WebPDecoder::new(Cursor::new(bytes)).map_err(|_| unreadable())?;
    let (width, height) = decoder.dimensions();
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(unreadable());
    }
    let channels = if decoder.has_alpha() { 4 } else { 3 };
    let mut pixels = vec![0; decoder.output_buffer_size().ok_or_else(unreadable)?];
    decoder.read_image(&mut pixels).map_err(|_| unreadable())?;
    let (w, h, rgba) = shrink(&pixels, width as usize, height as usize, channels);
    Ok(base64::encode(&thumbhash::rgba_to_thumb_hash(w, h, &rgba)))
}

// The picture averaged down to at most `SIDE` pixels on each side, as RGBA.
fn shrink(pixels: &[u8], width: usize, height: usize, channels: usize) -> (usize, usize, Vec<u8>) {
    let scale = (SIDE as f64 / width.max(height) as f64).min(1.0);
    let w = ((width as f64 * scale).round() as usize).clamp(1, SIDE);
    let h = ((height as f64 * scale).round() as usize).clamp(1, SIDE);
    let mut rgba = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        let (top, bottom) = (y * height / h, ((y + 1) * height / h).max(y * height / h + 1));
        for x in 0..w {
            let (left, right) = (x * width / w, ((x + 1) * width / w).max(x * width / w + 1));
            let mut sum = [0u64; 4];
            for row in top..bottom {
                for pixel in pixels[(row * width + left) * channels..(row * width + right) * channels].chunks_exact(channels) {
                    for (total, value) in sum.iter_mut().zip(pixel) {
                        *total += u64::from(*value);
                    }
                    if channels == 3 {
                        sum[3] += 255;
                    }
                }
            }
            let count = ((bottom - top) * (right - left)) as u64;
            rgba.extend(sum.map(|total| (total / count) as u8));
        }
    }
    (w, h, rgba)
}

// A card's gallery picture without a hash yet: where the hash goes (the cover's `src`, or the blob's `path` of a card
// without a cover) and the WebP rendition below `/media/` it is read from.
struct Picture {
    source: String,
    rendition: String,
}

// A rendition size of the media grammar: `WxH`, then `-t` or `-c` or nothing.
fn is_size(size: &str) -> bool {
    let digits = |text: &str| !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit());
    let size = size.strip_suffix("-t").or_else(|| size.strip_suffix("-c")).unwrap_or(size);
    size.split_once('x').is_some_and(|(width, height)| digits(width) && digits(height))
}

// The WebP rendition of a picture below `/media/`: the cover's own when it is one, the still of a moving cover, or the
// original fitted to the cover box.
fn webp(relative: &str, blob: Option<&Value>) -> Result<String> {
    let (path, suffix) = relative.split_once(';').unwrap_or((relative, ""));
    if suffix.ends_with(".webp") {
        return Ok(relative.to_owned());
    }
    if let Some((size, _)) = suffix.rsplit_once('.').filter(|(size, _)| is_size(size)) {
        return Ok(format!("{path};{size}.webp"));
    }
    match blob {
        Some(blob) => Ok(cover(blob, 800.0, 1000.0)?.get("src").to_js_string().trim_start_matches("/media/").to_owned()),
        None => Ok(format!("{path};800x1000-t.webp")),
    }
}

fn picture(data: &Value, extras: &Value) -> Result<Option<Picture>> {
    let cover = extras.get("cover");
    if cover.truthy() {
        let Some(src) = cover.get("src").as_str() else { return Ok(None) };
        let Some(relative) = src.strip_prefix("/media/") else { return Ok(None) };
        if cover.get("thumbHash").is_string() || relative.is_empty() {
            return Ok(None);
        }
        return Ok(Some(Picture { source: src.to_owned(), rendition: webp(relative, None)? }));
    }
    let blob = data.get("blob");
    let Some(path) = blob.get("path").as_str().filter(|path| !path.is_empty() && !path.contains("://")) else { return Ok(None) };
    if blob.get("thumbHash").is_string() {
        return Ok(None);
    }
    Ok(Some(Picture { source: path.to_owned(), rendition: webp(path.trim_start_matches("/media/"), Some(blob))? }))
}

// The hash of a picture, `None` when the library has no such picture, or a status error for one it cannot render or
// read.
async fn hash(media: &dyn Media, rendition: &str) -> Result<Option<String>> {
    let Some(found) = media.response(rendition).await? else { return Ok(None) };
    let bytes = media.send(&found, "GET", None).await?.body.bytes().await?;
    of_webp(&bytes).map(Some)
}

/// Give a card's gallery picture its ThumbHash, read from a WebP rendition of the picture. A picture the library does
/// not have, cannot render or cannot read gets an empty `thumbHash`, so it is not tried again.
///
/// # Arguments
/// * `store` - The library.
/// * `writer` - The library's writer.
/// * `media` - The library's media, which makes the rendition when it has none.
/// * `id` - Card id.
///
/// # Returns
/// Whether the card took a hash; false for a card that is gone, has no picture on the library server or has its
/// `thumbHash` already, for a picture that cannot be hashed, and for one that changed while it was read.
///
/// # Errors
/// The media's or the database's error other than a 4xx status, such as an unreachable media container; the card is
/// left as it was.
pub async fn fill(store: &Store, writer: &Writer, media: &dyn Media, id: &str) -> Result<bool> {
    let Some(row) = store.card(id, None)? else { return Ok(false) };
    if row.deleted_ns.is_some() {
        return Ok(false);
    }
    let Some(picture) = picture(&json::parse(&row.data)?, &json::parse(&row.extras)?)? else { return Ok(false) };
    let hash = match hash(media, &picture.rendition).await {
        Ok(hash) => hash,
        Err(Error::Status { status: 400..500, .. } | Error::Invalid(_)) => None,
        Err(error) => return Err(error),
    };
    let kept = writer.run(|writer| writer.picture_hash(id, &picture.source, hash.as_deref().unwrap_or("")))?;
    Ok(kept && hash.is_some())
}

/// The newest visible cards whose gallery picture on the library server has no ThumbHash yet.
///
/// # Arguments
/// * `store` - The library.
/// * `limit` - Most ids returned.
///
/// # Errors
/// The database's error.
pub fn missing(store: &Store, limit: usize) -> Result<Vec<String>> {
    let rows = store.db().query(
        &format!(
            "SELECT id FROM cards WHERE user_id = ? AND deleted_ns IS NULL AND (flags & {GRID_HIDDEN_FLAGS}) = 0 AND (
               (json_extract(extras, '$.cover.src') LIKE '/media/%' AND json_extract(extras, '$.cover.thumbHash') IS NULL)
               OR (json_extract(extras, '$.cover') IS NULL AND json_extract(data, '$.blob.path') IS NOT NULL
                   AND json_extract(data, '$.blob.thumbHash') IS NULL))
             ORDER BY bumped_ns DESC LIMIT ?"
        ),
        &[store.default_user_id()?.into(), (limit as i64).into()],
    )?;
    Ok(rows.iter().filter_map(|row| row.text("id").map(str::to_owned)).collect())
}
