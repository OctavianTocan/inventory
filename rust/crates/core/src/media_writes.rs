//! Port of `lib/media-writes.mjs`: pasted files and image attachments, handled before the JSON mutation routes.

use std::rc::Rc;
use std::sync::LazyLock;

use mm_js::{Value, json, obj};
use regex::Regex;

use crate::capture_http::forbidden;
use crate::enrichment::Enrichment;
use crate::form_data::{self, FormValue};
use crate::host::{Media, MediaInput, MediaUser, Resource, ResourceOptions};
use crate::http::{Request, search_param};
use crate::store::{IdKind, Store};
use crate::store_write::Writer;
use crate::uploads::Uploads;
use crate::{Error, Result};

static ATTACHMENTS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^/objects/([^/]+)/attachments$").expect("pattern"));

const PASTE_LIMIT: usize = 64 * 1024 * 1024;

/// The services the media writes use (`options` of `createMediaWrites`, without `reply`).
pub struct MediaWrites {
    /// Open store.
    pub store: Rc<Store>,
    /// Transactional writer.
    pub writer: Rc<Writer>,
    /// Media storage.
    pub media: Rc<dyn Media>,
    /// Completed upload access.
    pub uploads: Rc<Uploads>,
    /// Post-commit enrichment queue.
    pub jobs: Enrichment,
    /// Validated HTTP transport for remote attachments.
    pub resource: Rc<dyn Resource>,
    /// Permit only loopback remote attachments.
    pub offline: bool,
}

// A request value as node:sqlite binds it as the first parameter of a lookup by id: an object or array is read as
// named parameters, so an empty one leaves the id unbound and a key it names throws a plain `Error`.
pub(crate) fn bind_id(value: &Value) -> Result<String> {
    match value {
        Value::String(id) => Ok(id.clone()),
        Value::Number(_) => Ok(value.to_js_string()),
        Value::Null => Ok(String::new()),
        Value::Array(items) if items.is_empty() => Ok(String::new()),
        Value::Object(object) if object.is_empty() => Ok(String::new()),
        Value::Array(_) => Err(Error::internal("Unknown named parameter '0'")),
        Value::Object(object) => Err(Error::internal(format!("Unknown named parameter '{}'", object.keys().next().unwrap_or_default()))),
        _ => Err(Error::invalid("Provided value cannot be bound to SQLite parameter 1.")),
    }
}

/// The id a JS upload lookup binds to SQLite, after the sweep that runs before it (`uploads.prepare` and
/// `uploads.source`).
///
/// # Arguments
/// * `uploads` - Uploads whose sweep runs before the lookup.
/// * `value` - The request's `uploadId`.
///
/// # Returns
/// The id as text; an id that never matches for `null` and empty objects or arrays.
///
/// # Errors
/// After the sweep, a `TypeError` for a value SQLite cannot bind and an internal error for an object or array with
/// entries; the sweep's error.
pub async fn upload_id(uploads: &Uploads, value: &Value) -> Result<String> {
    let bound = bind_id(value);
    if bound.is_err() {
        uploads.sweep().await?;
    }
    bound
}

/// The library's account as `media.prepare` receives it (`store.user()`).
///
/// # Arguments
/// * `store` - Open store.
///
/// # Errors
/// A `TypeError` when the store has no user, or the database's error.
pub fn media_user(store: &Store) -> Result<MediaUser> {
    let user = store.user(None)?.ok_or_else(|| Error::invalid("Cannot read properties of undefined (reading 'accountUid')"))?;
    Ok(MediaUser { account_uid: user.account_uid, id_prefix: user.id_prefix })
}

// Read one bounded pasted file or attachment without passing its bytes through JSON.
async fn binary(request: &mut Request) -> Result<Vec<u8>> {
    request.take_body().bytes_within(PASTE_LIMIT).await.map_err(|error| match error.http_status() {
        Some(413) => Error::status(413, "Pasted file is too large; use chunked uploads"),
        _ => error,
    })
}

// `[...value ?? []]` of the attachments a card already has.
fn spread(value: &Value) -> Result<Vec<Value>> {
    match value {
        Value::Undefined | Value::Null => Ok(Vec::new()),
        Value::Array(items) => Ok(items.clone()),
        Value::String(text) => Ok(text.chars().map(|character| Value::String(character.to_string())).collect()),
        _ => Err(Error::invalid("baseline.extras.attachments is not iterable")),
    }
}

impl MediaWrites {
    /// Answer a clipboard file (`POST /objects` as multipart) or an image attachment
    /// (`POST /objects/:id/attachments`), as the function `createMediaWrites` returns does.
    /// Attachments with `type=screenshot` also replace the cover and finish its pending display state.
    ///
    /// # Arguments
    /// * `request` - The request; its body is read only for those two routes.
    ///
    /// # Returns
    /// The status and JSON body to send with the quota headers, or `None` for any other request.
    ///
    /// # Errors
    /// Status 400, 404, 413 and 415 errors with the JS messages, `TypeError`s for malformed requests, and the
    /// media's, uploads', resource's and writer's errors.
    pub async fn handle(&self, request: &mut Request) -> Result<Option<(u16, Value)>> {
        if request.method != "POST" {
            return Ok(None);
        }
        let mime = request.header("content-type").unwrap_or("").to_owned();
        let pathname = request.url.path().to_owned();
        if pathname == "/objects" && mime.starts_with("multipart/form-data") {
            return self.paste(request, &mime).await.map(Some);
        }
        let Some(found) = ATTACHMENTS.captures(&pathname) else { return Ok(None) };
        self.attach(request, &mime, &found[1]).await.map(Some)
    }

    async fn paste(&self, request: &mut Request, mime: &str) -> Result<(u16, Value)> {
        let bytes = binary(request).await?;
        let invalid = || Error::status(400, "Invalid pasted file form");
        let form = form_data::parse(mime, &bytes).map_err(|_| invalid())?;
        let metadata = match form_data::get(&form, "metadata") {
            Some(FormValue::Text(text)) if !text.is_empty() => json::parse(text).map_err(|_| invalid())?,
            Some(FormValue::File(file)) => json::parse(&mm_js::text::decode_utf8(&file.bytes)).map_err(|_| invalid())?,
            _ => obj! {}.into(),
        };
        let Some(FormValue::File(file)) = form_data::get(&form, "blob") else { return Err(Error::status(400, "Pasted file is required")) };
        let id = self.store.new_id(IdKind::Card, None)?;
        let prepared = self.media.prepare(MediaInput::Bytes(file.bytes.clone()), &file.content_type, &file.name, &media_user(&self.store)?, &id).await?;
        let media = Value::from(obj! { "data" => prepared.data, "extras" => prepared.extras, "id" => id.as_str() });
        let result = self.writer.run(|writer| writer.create(&metadata, &media))?;
        self.jobs.schedule(&id, false);
        Ok((result.get("status").to_number() as u16, result.get("body").clone()))
    }

    async fn attach(&self, request: &mut Request, mime: &str, id: &str) -> Result<(u16, Value)> {
        if self.store.card(id, None)?.is_none() {
            return Err(Error::status(404, "Card not found"));
        }
        let mut input = MediaInput::Bytes(binary(request).await?);
        let (mut kind, mut name, mut upload) = (mime.to_owned(), "attachment".to_owned(), None);
        if mime.starts_with("application/json") {
            let MediaInput::Bytes(bytes) = &input else { unreachable!() };
            let body = json::parse(&String::from_utf8_lossy(bytes)).map_err(|_| Error::status(400, "Invalid attachment request"))?;
            if body.is_nullish() {
                return Err(Error::invalid("Cannot read properties of null (reading 'uploadId')"));
            }
            if body.get("uploadId").truthy() {
                let key = upload_id(&self.uploads, body.get("uploadId")).await?;
                let source = self.uploads.source(&key).await?;
                (input, kind, name, upload) = (source.input, source.content_type, source.name, Some((key, body.get("uploadId").clone())));
            } else if body.get("remoteUrl").truthy() {
                let Some(url) = body.get("remoteUrl").as_str() else { return Err(forbidden()) };
                let remote = self.resource.fetch(url, ResourceOptions { offline: self.offline, limit: Some(25 * 1024 * 1024) }).await?;
                (input, kind) = (MediaInput::Bytes(remote.bytes), remote.mime);
            } else {
                return Err(Error::status(400, "An uploadId or remoteUrl is required"));
            }
        }
        let prepared = self.media.prepare(input, &kind, &name, &media_user(&self.store)?, id).await?;
        if prepared.data.get("type").as_str() != Some("Image") {
            return Err(Error::status(415, "Attachments must be images"));
        }
        let blob = prepared.data.get("blob");
        if blob.is_nullish() {
            return Err(Error::invalid("Cannot read properties of undefined (reading 'path')"));
        }
        let attachment = obj! { "path" => format!("/media/{}", blob.get("path").to_js_string()), "width" => blob.get("width").clone(), "height" => blob.get("height").clone() };
        let current = self.store.card(id, None)?.ok_or_else(|| Error::invalid("Cannot read properties of undefined (reading 'data')"))?;
        let mut baseline = Value::from(obj! { "data" => json::parse(&current.data)?, "extras" => json::parse(&current.extras)? });
        let mut attachments = spread(baseline.get("extras").get("attachments"))?;
        let mut added = attachment.clone();
        if let Some((_, original)) = &upload {
            added.insert("uploadId", original.clone());
        }
        attachments.push(added.into());
        let mut patch = Value::from(obj! { "extras" => obj! { "attachments" => Value::Array(attachments) } });
        if search_param(&request.url, "type").as_deref() == Some("screenshot") {
            let extras = patch.get_mut("extras").expect("extras object");
            extras.set("cover", self.media.cover(blob)?);
            for key in ["capturePending", "screenshotPending", "screenshotFailed"] {
                extras.set(key, Value::Undefined);
            }
            let data = baseline.get("data");
            let mut changes = obj! { "flags" => (data.get("flags").to_number() as i32) & !((1 << 21) | (1 << 22)) };
            if data.get("type").as_str() == Some("Placeholder") {
                changes.insert("type", "WebPage");
            }
            patch.set("data", changes);
        }
        self.writer.run(|writer| {
            writer.enrich(id, &patch, &mut baseline)?;
            if let Some((key, _)) = &upload {
                self.uploads.claim(key, id, true)?;
            }
            Ok(())
        })?;
        if let Some((key, _)) = &upload {
            self.uploads.cleanup_claimed(key).await?;
        }
        Ok((200, attachment.into()))
    }
}
