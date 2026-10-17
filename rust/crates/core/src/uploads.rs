//! Port of `lib/uploads.mjs`: resumable chunked uploads with bounded temporary storage.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::LazyLock;
use std::task::{Context, Poll};

use futures_util::StreamExt;
use mm_js::{Value, base64, json, text, uri};
use regex::Regex;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::host::{Media, MediaInput, MediaUser, UploadStorage};
use crate::http::{Body, ByteStream, Headers};
use crate::sql::{Row, Sql, SqlExt};
use crate::store::{IdKind, Store, User};
use crate::{Error, Result, obj, params, runtime};

const DEFAULT_MAX_BYTES: u64 = 1024 * 1024 * 1024;
const DAY_MS: f64 = 24.0 * 60.0 * 60.0 * 1000.0;
const SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

static BEARER: LazyLock<Regex> = LazyLock::new(|| Regex::new("^Bearer ([^\n\r\u{2028}\u{2029}]+)$").expect("pattern"));
static RANGE: LazyLock<Regex> = LazyLock::new(|| Regex::new("^bytes ([0-9]+)-([0-9]+)/([0-9]+)$").expect("pattern"));
static UPLOAD_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[a-f0-9]{22}$").expect("pattern"));
static MEDIA_PREFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^/media/").expect("pattern"));

#[derive(Default)]
struct Coordinator {
    busy: HashSet<String>,
    reservations: HashMap<String, u64>,
    sweeping: bool,
}

thread_local! {
    static COORDINATORS: RefCell<HashMap<String, Rc<RefCell<Coordinator>>>> = RefCell::new(HashMap::new());
}

/// Tunables of [`Uploads::new`] (`options` of `createUploads`, plus the random source).
#[derive(Clone, Copy)]
pub struct UploadOptions {
    /// Largest persisted plus reserved temporary byte count across uploads; at least 1.
    pub max_bytes: u64,
    /// Random bytes for upload ids and tokens.
    pub random: fn(usize) -> Vec<u8>,
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self { max_bytes: DEFAULT_MAX_BYTES, random: runtime::random_bytes }
    }
}

/// A chunk request: the headers and the body of one `PUT /uploads/:id`.
pub struct UploadRequest {
    /// Request headers, looked up case-insensitively.
    pub headers: Headers,
    /// The chunk's bytes.
    pub body: Body,
}

/// Where a finished upload's bytes are read from (`source`).
pub struct UploadSource {
    /// The temporary chunks, or the permanent original of a claimed upload.
    pub input: MediaInput,
    /// Content type of the file.
    pub content_type: String,
    /// Original file name.
    pub name: String,
}

struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            return Poll::Ready(());
        }
        self.0 = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

fn fail<T>(status: u16, message: &str) -> Result<T> {
    Err(Error::status(status, message))
}

fn sha256(text: &str) -> Vec<u8> {
    Sha256::digest(text.as_bytes()).to_vec()
}

fn exactly(body: Body, length: u64) -> ByteStream {
    let state = (body.into_stream(), 0u64, false);
    Box::pin(futures_util::stream::unfold(state, move |(mut stream, seen, finished)| async move {
        if finished {
            return None;
        }
        let mismatch = || Some((Err(Error::status(400, "Chunk length mismatch")), (Box::pin(futures_util::stream::empty()) as ByteStream, 0, true)));
        match stream.next().await {
            Some(Ok(chunk)) => {
                let seen = seen + chunk.len() as u64;
                if seen > length {
                    return mismatch();
                }
                Some((Ok(chunk), (stream, seen, false)))
            }
            Some(Err(error)) => Some((Err(error), (stream, seen, true))),
            None if seen != length => mismatch(),
            None => None,
        }
    }))
}

fn int(row: &Row, column: &str) -> f64 {
    row.number(column).unwrap_or(0.0)
}

fn safe(number: f64) -> bool {
    number.is_finite() && number.fract() == 0.0 && number.abs() <= SAFE_INTEGER
}

/// Resumable chunk uploads for one user's open store (`createUploads`).
pub struct Uploads {
    store: Rc<Store>,
    db: Rc<dyn Sql>,
    user: User,
    storage: Rc<dyn UploadStorage>,
    media: Rc<dyn Media>,
    max_bytes: u64,
    random: fn(usize) -> Vec<u8>,
    state: Rc<RefCell<Coordinator>>,
}

impl Uploads {
    /// `createUploads`: build the upload operations of one library.
    ///
    /// # Arguments
    /// * `store` - Current user's open store.
    /// * `storage` - Chunk storage; uploads sharing a storage `key` share one coordinator on this thread.
    /// * `media` - Media handle for validating completed files and resolving claimed originals.
    /// * `options` - Temporary storage limit and random source.
    ///
    /// # Errors
    /// An invalid-data error when `max_bytes` is below 1 or the store has no user.
    pub fn new(store: Rc<Store>, storage: Rc<dyn UploadStorage>, media: Rc<dyn Media>, options: UploadOptions) -> Result<Self> {
        if options.max_bytes < 1 || options.max_bytes as f64 > SAFE_INTEGER {
            return Err(Error::invalid("Invalid temporary upload limit"));
        }
        let user = store.user(None)?.ok_or_else(|| Error::invalid("Cannot read properties of null (reading 'id')"))?;
        let state = COORDINATORS.with(|map| map.borrow_mut().entry(storage.key()).or_default().clone());
        Ok(Self { db: store.db().clone(), store, user, storage, media, max_bytes: options.max_bytes, random: options.random, state })
    }

    fn find(&self, id: &str) -> Result<Row> {
        let row = self.db.one("SELECT * FROM uploads WHERE id = ? AND user_id = ?", params![id, self.user.id.as_str()])?;
        match row {
            Some(row) if row.number("expires_ms").unwrap_or(0.0) > runtime::now_ms() => Ok(row),
            _ => fail(404, "Upload not found or expired"),
        }
    }

    fn is_busy(&self, id: &str) -> bool {
        self.state.borrow().busy.contains(id)
    }

    /// `sweep`: remove expired rows, orphaned files and consumed originals without disturbing active uploads.
    ///
    /// # Errors
    /// The database's or the storage's error.
    pub async fn sweep(&self) -> Result<()> {
        while self.state.borrow().sweeping {
            YieldNow(false).await;
        }
        self.state.borrow_mut().sweeping = true;
        let result = self.sweep_now().await;
        self.state.borrow_mut().sweeping = false;
        result
    }

    async fn sweep_now(&self) -> Result<()> {
        let rows = self.db.query("SELECT id, card_id, received, expires_ms FROM uploads", &[])?;
        let known: HashSet<String> = rows.iter().filter_map(|row| row.text("id").map(str::to_owned)).collect();
        for row in &rows {
            let id = row.text("id").unwrap_or("");
            if self.is_busy(id) {
                continue;
            }
            let expired = int(row, "expires_ms") <= runtime::now_ms();
            if row.text("card_id").is_some_and(|card| !card.is_empty()) || expired {
                self.storage.remove(id).await?;
            } else if self.storage.size(id).await?.is_some_and(|size| size as f64 > int(row, "received")) {
                self.storage.truncate(id, int(row, "received") as u64).await?;
            }
            if expired {
                self.db.execute("DELETE FROM uploads WHERE id = ?", params![id])?;
            }
        }
        for name in self.storage.names().await? {
            if UPLOAD_ID.is_match(&name) && !known.contains(&name) && !self.is_busy(&name) {
                self.storage.remove(&name).await?;
            }
        }
        Ok(())
    }

    fn reserve(&self, id: &str, length: u64) -> Result<()> {
        let used = self.db.one("SELECT COALESCE(SUM(received), 0) AS bytes FROM uploads WHERE card_id IS NULL", &[])?.map_or(0.0, |row| int(&row, "bytes"));
        let mut state = self.state.borrow_mut();
        let held: u64 = state.reservations.values().sum();
        if used + held as f64 + length as f64 > self.max_bytes as f64 {
            return fail(413, "Temporary upload storage limit exceeded");
        }
        state.reservations.insert(id.to_owned(), length);
        Ok(())
    }

    /// `authorize`: mint upload capabilities.
    ///
    /// # Arguments
    /// * `count` - Null for one upload, otherwise a whole number from 1 to 100.
    ///
    /// # Returns
    /// One `{ url, token }` object for a null count, otherwise an array of them.
    ///
    /// # Errors
    /// Status 400 `Upload count must be 1 to 100`, or the database's error.
    pub async fn authorize(&self, count: &Value) -> Result<Value> {
        self.sweep().await?;
        let wanted = if matches!(count, Value::Null) { 1.0 } else { count.to_number() };
        if wanted.fract() != 0.0 || !wanted.is_finite() || !(1.0..=100.0).contains(&wanted) {
            return fail(400, "Upload count must be 1 to 100");
        }
        let random = self.random;
        let results = self.store.transaction(|| {
            (0..wanted as usize)
                .map(|_| {
                    let id = base64::hex(&random(11));
                    let token = base64::hex(&random(32));
                    self.db.execute(
                        "INSERT INTO uploads (id, user_id, token_hash, expires_ms) VALUES (?, ?, ?, ?)",
                        params![id.as_str(), self.user.id.as_str(), base64::hex(&sha256(&token)), runtime::now_ms() + DAY_MS],
                    )?;
                    Ok(Value::from(obj! { "url" => format!("/uploads/{id}"), "token" => token }))
                })
                .collect::<Result<Vec<Value>>>()
        })?;
        Ok(if matches!(count, Value::Null) { results.into_iter().next().unwrap_or(Value::Undefined) } else { Value::Array(results) })
    }

    /// `put`: store one chunk of an upload.
    ///
    /// # Arguments
    /// * `id` - Upload id from [`Uploads::authorize`].
    /// * `request` - Headers (`authorization`, `content-range`, `content-type`, `x-file-name`, `x-upload-id`) and body.
    ///
    /// # Returns
    /// `{ id, blob: { id }, received, done }`.
    ///
    /// # Errors
    /// Status errors 400, 401, 404, 409 and 413 with the JS messages, or the storage's or database's error.
    pub async fn put(&self, id: &str, request: UploadRequest) -> Result<Value> {
        self.sweep().await?;
        let row = self.find(id)?;
        let header = |name: &str| request.headers.get(name).map(str::to_owned);
        let token = BEARER.captures(&header("authorization").unwrap_or_default()).map(|found| found[1].to_owned());
        let Some(token) = token else { return fail(401, "Invalid upload authorization") };
        let stored = base64::from_hex(row.text("token_hash").unwrap_or(""));
        let given = sha256(&token);
        if stored.len() != given.len() {
            return Err(Error::internal("Input buffers must have the same byte length"));
        }
        if !bool::from(given.ct_eq(&stored)) {
            return fail(401, "Invalid upload authorization");
        }
        if self.is_busy(id) {
            return fail(409, "Upload chunk already in progress");
        }
        if row.number("done").unwrap_or(0.0) != 0.0 {
            return fail(409, "Upload already complete");
        }
        if header("x-upload-id").is_some_and(|claimed| !claimed.is_empty() && claimed != id) {
            return fail(409, "Upload ID mismatch");
        }
        let range = header("content-range").unwrap_or_default();
        let Some(range) = RANGE.captures(&range) else { return fail(400, "Content-Range is required") };
        let number = |index: usize| range[index].parse::<f64>().unwrap_or(f64::INFINITY);
        let (start, end, total) = (number(1), number(2), number(3));
        let size = row.number("size");
        if ![start, end, total].iter().all(|value| safe(*value))
            || start != int(&row, "received")
            || end < start
            || end >= total
            || size.is_some_and(|size| total != size)
        {
            return fail(409, "Upload range mismatch");
        }
        let length = end - start + 1.0;
        if total > 500_000_000.0 || length > 32.0 * 1024.0 * 1024.0 {
            return fail(413, "Upload exceeds size limit");
        }
        let mime = header("content-type").unwrap_or_default().split(';').next().unwrap_or("").to_owned();
        let Ok(decoded) = uri::decode_uri(&header("x-file-name").unwrap_or_else(|| "upload".into())) else {
            return fail(400, "Invalid upload filename");
        };
        let name = text::slice(&decoded, 0, Some(255));
        if start != 0.0 && (row.text("mime") != Some(mime.as_str()) || row.text("name") != Some(name.as_str())) {
            return fail(409, "Upload metadata mismatch");
        }
        self.reserve(id, length as u64)?;
        self.state.borrow_mut().busy.insert(id.to_owned());
        let stored = async {
            self.storage.write(id, start as u64, Body::Stream(exactly(request.body, length as u64)), Some(length as u64)).await?;
            let done = end + 1.0 == total;
            self.db.execute(
                "UPDATE uploads SET name = ?, mime = ?, size = ?, received = ?, done = ? WHERE id = ?",
                params![name.as_str(), mime.as_str(), total, end + 1.0, if done { 1.0 } else { 0.0 }, id],
            )?;
            Ok(Value::from(obj! { "id" => id, "blob" => obj! { "id" => id }, "received" => end + 1.0, "done" => done }))
        }
        .await;
        let mut state = self.state.borrow_mut();
        state.reservations.remove(id);
        state.busy.remove(id);
        stored
    }

    /// `prepare`: identify a completed upload and make its card media.
    ///
    /// # Arguments
    /// * `id` - Upload id.
    ///
    /// # Returns
    /// `{ existingId }` for a consumed upload, otherwise the prepared `{ data, extras, id }`.
    ///
    /// # Errors
    /// Status 404 or 409, the media's status errors, or 422 `Media preview could not be generated`.
    pub async fn prepare(&self, id: &str) -> Result<Value> {
        self.sweep().await?;
        let row = self.find(id)?;
        if row.number("done").unwrap_or(0.0) == 0.0 || row.number("received") != row.number("size") {
            return fail(409, "Upload is incomplete");
        }
        if let Some(card) = row.text("card_id").filter(|card| !card.is_empty()) {
            return Ok(obj! { "existingId" => card }.into());
        }
        if self.is_busy(id) {
            return fail(409, "Upload is being processed");
        }
        self.state.borrow_mut().busy.insert(id.to_owned());
        let made: Result<Value> = async {
            let card_id = self.store.new_id(IdKind::Card, None)?;
            let media_user = MediaUser { account_uid: self.user.account_uid.clone(), id_prefix: self.user.id_prefix.clone() };
            let prepared = self.media.prepare(self.storage.input(id), row.text("mime").unwrap_or(""), row.text("name").unwrap_or(""), &media_user, &card_id).await?;
            Ok(Value::from(obj! { "data" => prepared.data, "extras" => prepared.extras, "id" => card_id }))
        }
        .await;
        self.state.borrow_mut().busy.remove(id);
        match made {
            Err(error) if error.http_status().is_none() => fail(422, "Media preview could not be generated"),
            other => other,
        }
    }

    /// `claim`: bind an upload to the card made from it.
    ///
    /// # Arguments
    /// * `id` - Upload id.
    /// * `card_id` - Card that consumes the upload.
    /// * `reuse` - Accept an upload already consumed by any card.
    ///
    /// # Errors
    /// Status 404, or 409 `Upload already consumed`.
    pub fn claim(&self, id: &str, card_id: &str, reuse: bool) -> Result<()> {
        let row = self.find(id)?;
        let owner = row.text("card_id").filter(|card| !card.is_empty());
        if owner.is_some() && reuse {
            return Ok(());
        }
        if owner.is_some_and(|owner| owner != card_id) {
            return fail(409, "Upload already consumed");
        }
        self.db.execute("UPDATE uploads SET card_id = ? WHERE id = ?", params![card_id, id])?;
        Ok(())
    }

    /// `cleanupClaimed`: delete the temporary copy of a consumed upload.
    ///
    /// # Arguments
    /// * `id` - Upload id.
    ///
    /// # Errors
    /// The database's or the storage's error.
    pub async fn cleanup_claimed(&self, id: &str) -> Result<()> {
        let row = self.db.one("SELECT card_id FROM uploads WHERE id = ? AND user_id = ?", params![id, self.user.id.as_str()])?;
        if row.is_some_and(|row| row.text("card_id").is_some_and(|card| !card.is_empty())) {
            self.storage.remove(id).await?;
        }
        Ok(())
    }

    /// `source`: where the bytes of a completed upload can be read.
    ///
    /// # Arguments
    /// * `id` - Upload id.
    ///
    /// # Returns
    /// The temporary chunks, or the card's permanent original once the upload is claimed.
    ///
    /// # Errors
    /// Status 404 or 409, including 404 `Upload media is unavailable`.
    pub async fn source(&self, id: &str) -> Result<UploadSource> {
        self.sweep().await?;
        let row = self.find(id)?;
        if row.number("done").unwrap_or(0.0) == 0.0 || row.number("received") != row.number("size") {
            return fail(409, "Upload is incomplete");
        }
        let content_type = row.text("mime").unwrap_or("").to_owned();
        let name = row.text("name").unwrap_or("").to_owned();
        let Some(card_id) = row.text("card_id").filter(|card| !card.is_empty()) else {
            return Ok(UploadSource { input: self.storage.input(id), content_type, name });
        };
        let (mut data, mut extras) = (Value::Object(obj! {}), Value::Object(obj! {}));
        if let Some(card) = self.store.card(card_id, None)? {
            let parse = |text: &str| json::parse(text).map_err(|error| Error::internal(error.to_string()));
            (data, extras) = (parse(&card.data)?, parse(&card.extras)?);
        }
        let attachments = extras.get("attachments");
        let attachment = match attachments {
            Value::Array(items) => items.iter().find(|item| matches!(item.get("uploadId"), Value::String(found) if found == id)),
            other if other.is_nullish() => None,
            _ => return Err(Error::invalid("attachments.find is not a function")),
        };
        let attached = match attachment.map(|item| item.get("path")) {
            Some(Value::String(path)) => Some(MEDIA_PREFIX.replace(path, "").into_owned()),
            Some(other) if !other.is_nullish() => return Err(Error::invalid("path.replace is not a function")),
            _ => None,
        };
        let path = match attached {
            Some(path) => Value::String(path),
            None => data.get("blob").get("path").clone(),
        };
        let found = if path.truthy() { self.media.response(&path.to_js_string()).await? } else { None };
        let Some(found) = found else { return fail(404, "Upload media is unavailable") };
        Ok(UploadSource { input: self.storage.claimed_input(&found), content_type: found.content_type, name })
    }
}
