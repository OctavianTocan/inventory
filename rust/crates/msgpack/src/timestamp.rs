/// The extension type of MessagePack timestamps.
pub const EXT_TIMESTAMP: i8 = -1;

const TIMESTAMP32_MAX_SEC: f64 = 4_294_967_295.0;
const TIMESTAMP64_MAX_SEC: f64 = 17_179_869_183.0;

/// `encodeTimeSpecToTimestamp({ sec, nsec })`: the timestamp extension's data, in its 4, 8 or 12 byte form.
///
/// # Arguments
/// * `sec` - Whole seconds since the epoch.
/// * `nsec` - Nanoseconds within the second, below 1e9.
pub fn timestamp(sec: f64, nsec: f64) -> Vec<u8> {
    if sec >= 0.0 && nsec >= 0.0 && sec <= TIMESTAMP64_MAX_SEC {
        if nsec == 0.0 && sec <= TIMESTAMP32_MAX_SEC {
            return (sec as u32).to_be_bytes().to_vec();
        }
        let high = to_int32(sec / 4_294_967_296.0) as u32 & 3;
        let mut data = ((to_int32(nsec) as u32) << 2 | high).to_be_bytes().to_vec();
        data.extend_from_slice(&(to_int32(sec) as u32).to_be_bytes());
        return data;
    }
    let mut data = to_uint32(nsec).to_be_bytes().to_vec();
    data.extend_from_slice(&to_uint32((sec / 4_294_967_296.0).floor()).to_be_bytes());
    data.extend_from_slice(&to_uint32(sec).to_be_bytes());
    data
}

/// The timestamp extension data `encode` writes for a `Date` holding `ms` milliseconds since the epoch.
pub fn timestamp_from_date(ms: f64) -> Vec<u8> {
    let sec = (ms / 1e3).floor();
    let nsec = (ms - sec * 1e3) * 1e6;
    let carry = (nsec / 1e9).floor();
    timestamp(sec + carry, nsec - carry * 1e9)
}

/// The `Date`, as milliseconds since the epoch (NaN when out of range), that `decode` reads from timestamp data.
///
/// # Errors
/// Data that is not 4, 8 or 12 bytes long, with the message the JS codec throws.
pub fn date_from_timestamp(data: &[u8]) -> Result<f64, String> {
    let word = |at: usize| f64::from(u32::from_be_bytes(data[at..at + 4].try_into().expect("four bytes")));
    let (sec, nsec) = match data.len() {
        4 => (word(0), 0.0),
        8 => {
            let first = u32::from_be_bytes(data[0..4].try_into().expect("four bytes"));
            (f64::from(first & 3) * 4_294_967_296.0 + word(4), f64::from(first >> 2))
        }
        12 => {
            let high = i32::from_be_bytes(data[4..8].try_into().expect("four bytes"));
            (f64::from(high) * 4_294_967_296.0 + word(8), word(0))
        }
        length => return Err(format!("Unrecognized data size for timestamp (expected 4, 8, or 12): {length}")),
    };
    let ms = sec * 1e3 + nsec / 1e6;
    Ok(if !ms.is_finite() || ms.abs() > 8.64e15 { f64::NAN } else { ms.trunc() + 0.0 })
}

fn to_uint32(number: f64) -> u32 {
    if !number.is_finite() {
        return 0;
    }
    number.trunc().rem_euclid(4_294_967_296.0) as u32
}

fn to_int32(number: f64) -> i32 {
    to_uint32(number) as i32
}
