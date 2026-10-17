//! Blocking image work over libvips (the build sharp ships, linked by `build.rs`), mirroring the pipeline sharp 0.34
//! drives for `lib/local-media.mjs`: format sniffing, metadata, the dominant colour and the auto-oriented, resized,
//! re-encoded renditions.

use std::ffi::{CStr, c_char, c_int, c_void};
use std::ptr::{null, null_mut};
use std::sync::Once;

type Raw = *mut c_void;

unsafe extern "C" {
    fn vips_init(argv0: *const c_char) -> c_int;
    fn vips_error_buffer() -> *const c_char;
    fn vips_error_clear();
    fn vips_cache_set_max(max: c_int);
    fn vips_concurrency_set(concurrency: c_int);
    fn g_log_set_handler(domain: *const c_char, levels: c_int, handler: extern "C" fn(*const c_char, c_int, *const c_char, Raw), data: Raw) -> u32;
    fn g_object_unref(object: Raw);
    fn g_free(memory: Raw);
    fn vips_call(operation: *const c_char, ...) -> c_int;
    fn vips_area_unref(area: Raw) -> c_int;
    fn vips_blob_get(blob: Raw, length: *mut usize) -> *const c_void;
    fn vips_blob_get_type() -> usize;
    fn vips_array_double_new(array: *const f64, n: c_int) -> Raw;
    fn vips_foreign_find_load_buffer(data: *const c_void, size: usize) -> *const c_char;
    fn vips_image_new_from_buffer(buf: *const c_void, len: usize, options: *const c_char, ...) -> Raw;
    fn vips_image_get_width(image: Raw) -> c_int;
    fn vips_image_get_height(image: Raw) -> c_int;
    fn vips_image_get_bands(image: Raw) -> c_int;
    fn vips_image_get_format(image: Raw) -> c_int;
    fn vips_image_get_interpretation(image: Raw) -> c_int;
    fn vips_image_hasalpha(image: Raw) -> c_int;
    fn vips_image_get_typeof(image: Raw, name: *const c_char) -> usize;
    fn vips_image_get_int(image: Raw, name: *const c_char, out: *mut c_int) -> c_int;
    fn vips_image_write_to_memory(image: Raw, size: *mut usize) -> Raw;
    fn vips_image_is_sequential(image: Raw) -> c_int;
    fn vips_image_copy_memory(image: Raw) -> Raw;
    fn vips_image_remove(image: Raw, name: *const c_char) -> c_int;
    fn vips_copy(image: Raw, out: *mut Raw, ...) -> c_int;
    fn vips_icc_transform(image: Raw, out: *mut Raw, profile: *const c_char, ...) -> c_int;
    fn vips_flatten(image: Raw, out: *mut Raw, ...) -> c_int;
    fn vips_premultiply(image: Raw, out: *mut Raw, ...) -> c_int;
    fn vips_unpremultiply(image: Raw, out: *mut Raw, ...) -> c_int;
    fn vips_cast(image: Raw, out: *mut Raw, format: c_int, ...) -> c_int;
    fn vips_resize(image: Raw, out: *mut Raw, scale: f64, ...) -> c_int;
    fn vips_rot(image: Raw, out: *mut Raw, angle: c_int, ...) -> c_int;
    fn vips_flip(image: Raw, out: *mut Raw, direction: c_int, ...) -> c_int;
    fn vips_extract_area(image: Raw, out: *mut Raw, left: c_int, top: c_int, width: c_int, height: c_int, ...) -> c_int;
    fn vips_extract_band(image: Raw, out: *mut Raw, band: c_int, ...) -> c_int;
    fn vips_colourspace(image: Raw, out: *mut Raw, space: c_int, ...) -> c_int;
    fn vips_hist_find_ndim(image: Raw, out: *mut Raw, ...) -> c_int;
    fn vips_jpegsave_buffer(image: Raw, buf: *mut Raw, len: *mut usize, ...) -> c_int;
    fn vips_webpsave_buffer(image: Raw, buf: *mut Raw, len: *mut usize, ...) -> c_int;
}

const FAIL_ON_WARNING: c_int = 3;
const LOG_LEVEL_WARNING: c_int = 1 << 4;
const ACCESS_SEQUENTIAL: c_int = 1;
const INTERPRETATION_CMYK: c_int = 15;
const INTERPRETATION_LABS: c_int = 21;
const INTERPRETATION_SRGB: c_int = 22;
const INTERPRETATION_RGB16: c_int = 25;
const INTERPRETATION_GREY16: c_int = 26;
const FORMAT_USHORT: c_int = 2;
const KERNEL_LANCZOS3: c_int = 5;
const DIRECTION_HORIZONTAL: c_int = 0;
const PNG_FILTER_NONE: c_int = 0x08;
const SUBSAMPLE_ON: c_int = 1;

/// Result of a libvips call; the error is libvips' own message.
pub type Outcome<T> = Result<T, String>;

static INIT: Once = Once::new();

/// Start libvips once per process; every other function here calls it first.
pub fn init() {
    INIT.call_once(|| {
        // SAFETY: called once, with a NUL-terminated program name.
        let started = unsafe { vips_init(c"mymind".as_ptr()) };
        assert_eq!(started, 0, "libvips failed to start");
        // SAFETY: plain setters after init.
        unsafe { vips_cache_set_max(0) };
        // sharp runs libvips on one thread with glibc's allocator, to limit memory fragmentation.
        if cfg!(all(target_os = "linux", target_env = "gnu")) {
            unsafe { vips_concurrency_set(1) };
        }
        // sharp keeps libvips warnings (such as a truncated image's tile errors) out of the log.
        unsafe { g_log_set_handler(c"VIPS".as_ptr(), LOG_LEVEL_WARNING, ignore_log, null_mut()) };
    });
}

/// A GLib log handler that drops the message.
extern "C" fn ignore_log(_domain: *const c_char, _level: c_int, _message: *const c_char, _data: Raw) {}

struct Img(Raw);

impl Drop for Img {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: owns one reference to a live VipsImage.
            unsafe { g_object_unref(self.0) };
        }
    }
}

impl Img {
    fn width(&self) -> i32 {
        unsafe { vips_image_get_width(self.0) }
    }

    fn height(&self) -> i32 {
        unsafe { vips_image_get_height(self.0) }
    }

    fn bands(&self) -> i32 {
        unsafe { vips_image_get_bands(self.0) }
    }

    fn format(&self) -> c_int {
        unsafe { vips_image_get_format(self.0) }
    }

    fn interpretation(&self) -> c_int {
        unsafe { vips_image_get_interpretation(self.0) }
    }

    fn has_alpha(&self) -> bool {
        unsafe { vips_image_hasalpha(self.0) != 0 }
    }

    fn is_16_bit(&self) -> bool {
        matches!(self.interpretation(), INTERPRETATION_RGB16 | INTERPRETATION_GREY16)
    }

    fn has_profile(&self) -> bool {
        unsafe { vips_image_get_typeof(self.0, c"icc-profile-data".as_ptr()) == vips_blob_get_type() }
    }

    fn orientation(&self) -> i32 {
        let mut value: c_int = 0;
        // SAFETY: the field name is NUL-terminated and `value` outlives the call.
        unsafe {
            if vips_image_get_typeof(self.0, c"orientation".as_ptr()) != 0 && vips_image_get_int(self.0, c"orientation".as_ptr(), &raw mut value) == 0 {
                value
            } else {
                0
            }
        }
    }
}

fn take_error() -> String {
    // SAFETY: libvips returns a NUL-terminated buffer it owns.
    let text = unsafe { CStr::from_ptr(vips_error_buffer()).to_string_lossy().trim().to_owned() };
    unsafe { vips_error_clear() };
    if text.is_empty() { "libvips failed".to_owned() } else { text }
}

fn produce(call: impl FnOnce(*mut Raw) -> c_int) -> Outcome<Img> {
    let mut image: Raw = null_mut();
    if call(&raw mut image) != 0 || image.is_null() {
        return Err(take_error());
    }
    Ok(Img(image))
}

fn encoded(call: impl FnOnce(*mut Raw, *mut usize) -> c_int) -> Outcome<Vec<u8>> {
    let (mut buffer, mut length): (Raw, usize) = (null_mut(), 0);
    if call(&raw mut buffer, &raw mut length) != 0 || buffer.is_null() {
        return Err(take_error());
    }
    // SAFETY: libvips filled `buffer` with `length` bytes it allocated with g_malloc.
    let bytes = unsafe { std::slice::from_raw_parts(buffer.cast::<u8>(), length).to_vec() };
    unsafe { g_free(buffer) };
    Ok(bytes)
}

/// The formats sharp's bundled libvips can read, by the name `metadata()` reports.
fn kind(data: &[u8]) -> Option<&'static str> {
    // SAFETY: the pointer and length describe `data`.
    let name = unsafe { vips_foreign_find_load_buffer(data.as_ptr().cast(), data.len()) };
    if name.is_null() {
        unsafe { vips_error_clear() };
        return None;
    }
    let name = unsafe { CStr::from_ptr(name) }.to_str().ok()?;
    let rest = name.strip_prefix("VipsForeignLoad")?;
    [("Jpeg", "jpeg"), ("Png", "png"), ("Webp", "webp"), ("Tiff", "tiff"), ("Nsgif", "gif"), ("Gif", "gif"), ("Heif", "heif"), ("Svg", "svg")]
        .into_iter()
        .find(|(prefix, _)| rest.starts_with(prefix))
        .map(|(_, kind)| kind)
}

enum Load {
    Plain,
    Shrink(c_int),
    Scale(f64),
}

fn load(data: &[u8], how: &Load) -> Outcome<Img> {
    let (pointer, length) = (data.as_ptr().cast::<c_void>(), data.len());
    let none = c"".as_ptr();
    let end = null::<c_char>();
    // SAFETY: option names are NUL-terminated and the value types match what each option takes.
    let image = unsafe {
        match how {
            Load::Plain => vips_image_new_from_buffer(pointer, length, none, c"fail_on".as_ptr(), FAIL_ON_WARNING, c"access".as_ptr(), ACCESS_SEQUENTIAL, end),
            Load::Shrink(factor) => vips_image_new_from_buffer(
                pointer, length, none, c"fail_on".as_ptr(), FAIL_ON_WARNING, c"access".as_ptr(), ACCESS_SEQUENTIAL, c"shrink".as_ptr(), *factor, end,
            ),
            Load::Scale(scale) => vips_image_new_from_buffer(
                pointer, length, none, c"fail_on".as_ptr(), FAIL_ON_WARNING, c"access".as_ptr(), ACCESS_SEQUENTIAL, c"scale".as_ptr(), *scale, end,
            ),
        }
    };
    if image.is_null() { Err(take_error()) } else { Ok(Img(image)) }
}

/// A sequentially read image copied to memory, so a rotation can read it in any order (sharp's `StaySequential`).
fn in_memory(image: Img) -> Outcome<Img> {
    // SAFETY: `image` holds a live reference; the copies are new images this function owns.
    unsafe {
        if vips_image_is_sequential(image.0) == 0 {
            return Ok(image);
        }
        let memory = vips_image_copy_memory(image.0);
        if memory.is_null() {
            return Err(take_error());
        }
        let memory = Img(memory);
        let copy = produce(|out| vips_copy(memory.0, out, null::<c_char>()))?;
        vips_image_remove(copy.0, c"vips-sequential".as_ptr());
        Ok(copy)
    }
}

fn open(data: &[u8], limit: u64) -> Outcome<(Img, &'static str)> {
    init();
    let Some(kind) = kind(data) else { return Err("Input buffer contains unsupported image format".to_owned()) };
    let image = load(data, &Load::Plain)?;
    if limit > 0 && u64::from(image.width().unsigned_abs()) * u64::from(image.height().unsigned_abs()) > limit {
        return Err("Input image exceeds pixel limit".to_owned());
    }
    Ok((image, kind))
}

/// What sharp's `metadata()` reports of an image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Metadata {
    /// Format name: `jpeg`, `png`, `webp`, `gif`, `tiff`, `heif` or `svg`.
    pub format: &'static str,
    /// Stored width, before any EXIF rotation.
    pub width: i32,
    /// Stored height, before any EXIF rotation.
    pub height: i32,
    /// EXIF orientation, or 0 when the image has none.
    pub orientation: i32,
}

/// Identify an image the way `sharp(input).metadata()` does.
///
/// # Arguments
/// * `data` - The file's bytes.
/// * `limit` - Largest accepted pixel count; 0 for no limit.
///
/// # Errors
/// libvips' message when the format is unsupported, the header is invalid or the pixel limit is exceeded.
pub fn metadata(data: &[u8], limit: u64) -> Outcome<Metadata> {
    let (image, format) = open(data, limit)?;
    Ok(Metadata { format, width: image.width(), height: image.height(), orientation: image.orientation() })
}

/// The dominant colour sharp's `stats()` reports.
///
/// # Arguments
/// * `data` - The file's bytes.
/// * `limit` - Largest accepted pixel count; 0 for no limit.
///
/// # Returns
/// `#rrggbb` of the centre of the most populated cell of a 16 by 16 by 16 sRGB histogram.
///
/// # Errors
/// libvips' message when the image cannot be read.
pub fn dominant(data: &[u8], limit: u64) -> Outcome<String> {
    let (mut image, _) = open(data, limit)?;
    if image.has_alpha() {
        let bands = image.bands() - 1;
        image = produce(|out| unsafe { vips_extract_band(image.0, out, 0, c"n".as_ptr(), bands, null::<c_char>()) })?;
    }
    let image = produce(|out| unsafe { vips_colourspace(image.0, out, INTERPRETATION_SRGB, null::<c_char>()) })?;
    let hist = produce(|out| unsafe { vips_hist_find_ndim(image.0, out, c"bins".as_ptr(), 16 as c_int, null::<c_char>()) })?;
    let mut size = 0usize;
    // SAFETY: `hist` is live; libvips returns a g_malloc'd buffer of `size` bytes.
    let memory = unsafe { vips_image_write_to_memory(hist.0, &raw mut size) };
    if memory.is_null() {
        return Err(take_error());
    }
    let bands = hist.bands() as usize;
    let cells: Vec<u32> = unsafe { std::slice::from_raw_parts(memory.cast::<u8>(), size) }
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| u32::from_ne_bytes(*chunk))
        .collect();
    unsafe { g_free(memory) };
    let (mut best, mut at) = (0u32, 0usize);
    for (index, value) in cells.iter().enumerate() {
        if *value > best {
            (best, at) = (*value, index);
        }
    }
    let (pixel, z) = (at / bands, at % bands);
    let (x, y) = (pixel % hist.width() as usize, pixel / hist.width() as usize);
    Ok(format!("#{:02x}{:02x}{:02x}", x * 16 + 8, y * 16 + 8, z * 16 + 8))
}

/// The encoder a rendition is written with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Jpeg,
    Webp,
    Png,
}

/// What `sharp(input).rotate()[.flatten(...)].resize(width, height, { fit })` becomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resize {
    /// Target width.
    pub width: i32,
    /// Target height.
    pub height: i32,
    /// `fit: 'cover'` (centre crop) rather than `'inside'`.
    pub cover: bool,
    /// Flatten any alpha onto white.
    pub flatten: bool,
    /// Allow enlarging; false is `withoutEnlargement`.
    pub enlarge: bool,
    /// Output encoder.
    pub format: Format,
}

fn resolve_shrink(width: i32, height: i32, target_width: i32, target_height: i32, cover: bool, enlarge: bool) -> (f64, f64) {
    let mut h = f64::from(width) / f64::from(target_width);
    let mut v = f64::from(height) / f64::from(target_height);
    let pick_small = cover;
    if pick_small == (h < v) {
        v = h;
    } else {
        h = v;
    }
    if !enlarge {
        h = h.max(1.0);
        v = v.max(1.0);
    }
    (h.min(f64::from(width)), v.min(f64::from(height)))
}

fn orientation_steps(orientation: i32) -> (c_int, bool) {
    match orientation {
        6 => (1, false),
        3 => (2, false),
        8 => (3, false),
        2 => (0, true),
        7 => (3, true),
        4 => (2, true),
        5 => (1, true),
        _ => (0, false),
    }
}

fn to_srgb_profile(image: Img) -> Outcome<Img> {
    let profile = if image.interpretation() == INTERPRETATION_RGB16 { c"p3" } else { c"srgb" };
    let depth: c_int = if image.is_16_bit() { 16 } else { 8 };
    if image.has_profile() && !matches!(image.interpretation(), INTERPRETATION_LABS | INTERPRETATION_GREY16) {
        let converted = produce(|out| unsafe {
            vips_icc_transform(image.0, out, profile.as_ptr(), c"embedded".as_ptr(), 1 as c_int, c"depth".as_ptr(), depth, c"intent".as_ptr(), 0 as c_int, null::<c_char>())
        });
        return Ok(converted.unwrap_or_else(|_| {
            take_error();
            image
        }));
    }
    if image.interpretation() == INTERPRETATION_CMYK {
        return produce(|out| unsafe {
            vips_icc_transform(image.0, out, profile.as_ptr(), c"input_profile".as_ptr(), c"cmyk".as_ptr(), c"intent".as_ptr(), 0 as c_int, null::<c_char>())
        });
    }
    Ok(image)
}

fn flatten_white(image: &Img) -> Outcome<Img> {
    let multiplier = if image.is_16_bit() { 256.0 } else { 1.0 };
    let background = [255.0 * multiplier; 3];
    // SAFETY: the array copies the three values; the call and the unref bracket its use.
    let array = unsafe { vips_array_double_new(background.as_ptr(), 3) };
    let result = produce(|out| unsafe { vips_flatten(image.0, out, c"background".as_ptr(), array, null::<c_char>()) });
    unsafe { vips_area_unref(array) };
    result
}

/// Re-encode an image the way a rendition or the analysis image is made.
///
/// # Arguments
/// * `data` - The original's bytes.
/// * `plan` - Target size, fit, background and encoder.
/// * `limit` - Largest accepted pixel count; 0 for no limit.
///
/// # Returns
/// The encoded bytes, with EXIF orientation applied and no metadata kept.
///
/// # Errors
/// libvips' message for an unreadable image or a failed step.
pub fn render(data: &[u8], plan: &Resize, limit: u64) -> Outcome<Vec<u8>> {
    let (mut image, kind) = open(data, limit)?;
    let (rotation, flop) = orientation_steps(image.orientation());
    let (mut target_width, mut target_height) = (plan.width, plan.height);
    if rotation == 1 || rotation == 3 {
        (target_width, target_height) = (target_height, target_width);
    }
    let (h, v) = resolve_shrink(image.width(), image.height(), target_width, target_height, plan.cover, plan.enlarge);
    let shrink = h.min(v);
    let reload = match kind {
        "jpeg" => {
            let mut factor = if shrink >= 8.0 { 8 } else if shrink >= 4.0 { 4 } else if shrink >= 2.0 { 2 } else { 1 };
            if factor > 1 && shrink as i32 == factor {
                factor /= 2;
            }
            (factor > 1).then_some(Load::Shrink(factor))
        }
        "webp" if shrink > 1.0 => Some(Load::Scale(1.0 / shrink)),
        "svg" => Some(Load::Scale(1.0 / shrink)),
        _ => None,
    };
    if let Some(how) = reload.filter(|how| !matches!(how, Load::Scale(scale) if *scale == 1.0)) {
        image = load(data, &how)?;
    }
    let (h, v) = resolve_shrink(image.width(), image.height(), target_width, target_height, plan.cover, plan.enlarge);
    image = to_srgb_profile(image)?;
    if plan.flatten && image.has_alpha() {
        image = flatten_white(&image)?;
    }
    let resize = h != 1.0 || v != 1.0;
    let premultiplied_format = image.format();
    let premultiply = image.has_alpha() && resize;
    if premultiply {
        image = produce(|out| unsafe { vips_premultiply(image.0, out, null::<c_char>()) })?;
        image = produce(|out| unsafe { vips_cast(image.0, out, premultiplied_format, null::<c_char>()) })?;
    }
    if resize {
        image = produce(|out| unsafe { vips_resize(image.0, out, 1.0 / h, c"vscale".as_ptr(), 1.0 / v, c"kernel".as_ptr(), KERNEL_LANCZOS3, null::<c_char>()) })?;
    }
    if rotation != 0 {
        image = in_memory(image)?;
        image = produce(|out| unsafe { vips_rot(image.0, out, rotation, null::<c_char>()) })?;
    }
    if flop {
        image = produce(|out| unsafe { vips_flip(image.0, out, DIRECTION_HORIZONTAL, null::<c_char>()) })?;
    }
    let (width, height) = (image.width(), image.height());
    if plan.cover && (width != plan.width || height != plan.height) {
        let (crop_width, crop_height) = (plan.width.min(width), plan.height.min(height));
        let (left, top) = ((width - crop_width + 1) / 2, (height - crop_height + 1) / 2);
        image = produce(|out| unsafe { vips_extract_area(image.0, out, left, top, crop_width, crop_height, null::<c_char>()) })?;
    }
    if premultiply {
        image = produce(|out| unsafe { vips_unpremultiply(image.0, out, null::<c_char>()) })?;
        image = produce(|out| unsafe { vips_cast(image.0, out, premultiplied_format, null::<c_char>()) })?;
    }
    if image.is_16_bit() {
        image = produce(|out| unsafe { vips_cast(image.0, out, FORMAT_USHORT, null::<c_char>()) })?;
    }
    if image.interpretation() != INTERPRETATION_SRGB {
        let source = image.interpretation();
        image = produce(|out| unsafe { vips_colourspace(image.0, out, INTERPRETATION_SRGB, c"source_space".as_ptr(), source, null::<c_char>()) })?;
    }
    encode(&image, plan.format)
}

fn encode(image: &Img, format: Format) -> Outcome<Vec<u8>> {
    let end = null::<c_char>();
    // SAFETY: option names are NUL-terminated and each value has the C type the option takes.
    unsafe {
        match format {
            Format::Jpeg => encoded(|buffer, length| {
                vips_jpegsave_buffer(
                    image.0, buffer, length, c"keep".as_ptr(), 0 as c_int, c"Q".as_ptr(), 80 as c_int, c"interlace".as_ptr(), 0 as c_int,
                    c"subsample_mode".as_ptr(), SUBSAMPLE_ON, c"trellis_quant".as_ptr(), 0 as c_int, c"quant_table".as_ptr(), 0 as c_int,
                    c"overshoot_deringing".as_ptr(), 0 as c_int, c"optimize_scans".as_ptr(), 0 as c_int, c"optimize_coding".as_ptr(), 1 as c_int, end,
                )
            }),
            Format::Webp => encoded(|buffer, length| {
                vips_webpsave_buffer(
                    image.0, buffer, length, c"keep".as_ptr(), 0 as c_int, c"Q".as_ptr(), 80 as c_int, c"lossless".as_ptr(), 0 as c_int,
                    c"near_lossless".as_ptr(), 0 as c_int, c"smart_subsample".as_ptr(), 0 as c_int, c"smart_deblock".as_ptr(), 0 as c_int,
                    c"preset".as_ptr(), 0 as c_int, c"effort".as_ptr(), 4 as c_int, c"min_size".as_ptr(), 0 as c_int,
                    c"mixed".as_ptr(), 0 as c_int, c"alpha_q".as_ptr(), 100 as c_int, end,
                )
            }),
            Format::Png => {
                // sharp's libvips saves PNG with libspng, which has no C wrapper, so the operation is called by name.
                let mut blob: Raw = null_mut();
                let failed = vips_call(
                    c"pngsave_buffer".as_ptr(), image.0, &raw mut blob, c"keep".as_ptr(), 0 as c_int, c"interlace".as_ptr(), 0 as c_int,
                    c"compression".as_ptr(), 6 as c_int, c"filter".as_ptr(), PNG_FILTER_NONE, c"palette".as_ptr(), 0 as c_int,
                    c"Q".as_ptr(), 100 as c_int, c"effort".as_ptr(), 7 as c_int, c"bitdepth".as_ptr(), if image.is_16_bit() { 16 } else { 8 } as c_int,
                    c"dither".as_ptr(), 1.0f64, end,
                );
                if failed != 0 || blob.is_null() {
                    return Err(take_error());
                }
                let mut length = 0;
                let data = vips_blob_get(blob, &raw mut length);
                let bytes = std::slice::from_raw_parts(data.cast::<u8>(), length).to_vec();
                vips_area_unref(blob);
                Ok(bytes)
            }
        }
    }
}

/// Whether libvips can read an image of this format at all, by the name [`metadata`] reports.
pub fn readable(data: &[u8]) -> bool {
    init();
    kind(data).is_some()
}

/// An image decoded to 8-bit pixels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decoded {
    /// Width in pixels.
    pub width: i32,
    /// Height in pixels.
    pub height: i32,
    /// Bands per pixel, alpha included.
    pub bands: i32,
    /// Interleaved samples, row by row.
    pub pixels: Vec<u8>,
}

/// Decode an image to 8-bit samples, as the tests compare renditions.
///
/// # Arguments
/// * `data` - The file's bytes.
///
/// # Errors
/// libvips' message when the image cannot be read.
pub fn decode(data: &[u8]) -> Outcome<Decoded> {
    let (image, _) = open(data, 0)?;
    let image = if image.format() == 0 { image } else { produce(|out| unsafe { vips_cast(image.0, out, 0, c"shift".as_ptr(), 1 as c_int, null::<c_char>()) })? };
    let mut size = 0usize;
    // SAFETY: `image` is live; libvips returns a g_malloc'd buffer of `size` bytes.
    let memory = unsafe { vips_image_write_to_memory(image.0, &raw mut size) };
    if memory.is_null() {
        return Err(take_error());
    }
    let pixels = unsafe { std::slice::from_raw_parts(memory.cast::<u8>(), size) }.to_vec();
    unsafe { g_free(memory) };
    Ok(Decoded { width: image.width(), height: image.height(), bands: image.bands(), pixels })
}
