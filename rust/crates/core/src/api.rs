//! Port of `lib/api.mjs`: the library API over a store. It answers the routes it owns and leaves every other request
//! to the recordings.

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

use async_trait::async_trait;
use mm_js::{Object, Value, date, json, number, obj, uri};
use mm_msgpack::{EXT_TIMESTAMP, EncodeOptions, encode_into, timestamp};
use regex::Regex;
use url::Url;

use crate::app_api::AppApi;
use crate::device::device_variant;
use crate::enrichment::{Enrichment, EnrichmentOptions};
use crate::host::{Fetch, Media, Resource, Screenshot, UploadStorage};
use crate::http::{Headers, Request, Response, search_param};
use crate::media_writes::{MediaWrites, bind_id, upload_id};
use crate::render::iphone::iphone_card;
use crate::render::markdown::card_markdown;
use crate::render::views::{render_detail, render_pinned, render_share};
use crate::search::Search;
use crate::site::LibraryApi;
use crate::sql::{SqlExt, SqlValue};
use crate::store::{Change, CardRow, ChangesOptions, Store, ns_from_iso};
use crate::store_write::Writer;
use crate::uploads::{UploadOptions, UploadRequest, Uploads};
use crate::{Error, Result, runtime};

const JSON_TYPE: &str = "application/json; charset=utf-8";
const MSGPACK: &str = "application/msgpack";
const HTML_TYPE: &str = "text/html; charset=utf-8";
const BODY_LIMIT: usize = 8 * 1024 * 1024;

static INLINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*\*[^*]+\*\*|`[^`]+`|\*[^*]+\*|\[[^\]]+\]\([^)]+\)").expect("pattern"));
static LINK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\[([^\]]+)\]\(([^)]+)\)$").expect("pattern"));
static HEADING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#{1,6} ").expect("pattern"));
static LIST_ITEM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:[-*] |[0-9]+\. )").expect("pattern"));
static RULE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^---+$").expect("pattern"));
static TOM_LIST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<div(?-u:\b)[^>]*class="[^"]*(?-u:\b)tom__list(?-u:\b)[^"]*"[^>]*>"#).expect("pattern"));
static DIV_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"</?div(?-u:\b)[^>]*>").expect("pattern"));
static CONFIG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(<script id="config" type="application/json">)((?s:.)*?)(</script>)"#).expect("pattern"));
static DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]+$").expect("pattern"));

static UPLOAD_PUT: LazyLock<Regex> = LazyLock::new(|| route(r"^/uploads/([a-z0-9]+)$"));
static ANALYZE: LazyLock<Regex> = LazyLock::new(|| route(r"^/cards/([^/]+)/analyze$"));
static ACHIEVEMENT: LazyLock<Regex> = LazyLock::new(|| route(r"^/achievements/([^/]+)$"));
static SHARE_PAGE: LazyLock<Regex> = LazyLock::new(|| route(r"^/shares/([^/]+)$"));
static QUERY_CHECK: LazyLock<Regex> = LazyLock::new(|| route(r"^/queries/([^/]+)/check$"));
static RELATED: LazyLock<Regex> = LazyLock::new(|| route(r"^/embeddings/related/([^/]+)$"));
static DETAIL: LazyLock<Regex> = LazyLock::new(|| route(r"^/cards/([^/]+)/(?:detail|reader)$"));
static MARKDOWN: LazyLock<Regex> = LazyLock::new(|| route(r"^/cards/([^/]+)/markdown$"));
static CARD: LazyLock<Regex> = LazyLock::new(|| route(r"^/cards/([^/]+)$"));
static SPACE: LazyLock<Regex> = LazyLock::new(|| route(r"^/spaces/([^/]+)$"));
static OBJECT_READ: LazyLock<Regex> = LazyLock::new(|| route(r"^/objects/([^/]+)(?:/(tags|content|revisions))?$"));
static DISMISS: LazyLock<Regex> = LazyLock::new(|| route(r"^/promotions/([^/]+)/dismiss$"));
static BATCH: LazyLock<Regex> = LazyLock::new(|| route(r"^/batch/(tag|delete|add-to-space)$"));
static OBJECT: LazyLock<Regex> = LazyLock::new(|| route(r"^/objects/([^/]+)$"));
static OBJECT_ACTION: LazyLock<Regex> =
    LazyLock::new(|| route(r"^/objects/([^/]+)/(prose|note|content|entity|commit|tags|restore|pin|unpin|complete|uncomplete)$"));
static ANNOTATION: LazyLock<Regex> = LazyLock::new(|| route(r"^/annotations/([^/]+)$"));
static MEMBERSHIP: LazyLock<Regex> = LazyLock::new(|| route(r"^/spaces/([^/]+)/objects/([^/]+)$"));
static EXCLUDE: LazyLock<Regex> = LazyLock::new(|| route(r"^/queries/([^/]+)/objects/([^/]+)/exclude$"));
static USER: LazyLock<Regex> = LazyLock::new(|| route(r"^/users/([^/]+)$"));
static ONBOARD: LazyLock<Regex> = LazyLock::new(|| route(r"^/onboard/flags/([0-9]+)/(set|unset)$"));
static REVISION_RESTORE: LazyLock<Regex> = LazyLock::new(|| route(r"^/objects/([^/]+)/revisions/([^/]+)/restore$"));
static SHARES: LazyLock<Regex> = LazyLock::new(|| route(r"^/cards/([^/]+)/shares$"));
static SHARE: LazyLock<Regex> = LazyLock::new(|| route(r"^/cards/([^/]+)/shares/([^/]+)$"));
static THUMBNAIL: LazyLock<Regex> = LazyLock::new(|| route(r"^/cards/([^/]+)/remove-ai-thumbnail$"));
static RESTORE: LazyLock<Regex> = LazyLock::new(|| route(r"^/objects/([^/]+)/restore$"));
static PROSE_WRITE: LazyLock<Regex> = LazyLock::new(|| route(r"^/objects/([^/]+)(?:/(?:prose|commit|content)|/revisions/[^/]+/restore)?$"));

fn route(pattern: &str) -> Regex {
    Regex::new(pattern).expect("pattern")
}

/// Broadcasts a committed hub event: `(target, payload)`.
pub type Push = Box<dyn Fn(&str, &Value)>;

/// Whether the server can replay this exact request.
pub type HasReplay = Box<dyn Fn(&Request) -> Result<bool>>;

/// What [`Api::new`] serves (`options` of `createApi` for an already open store).
pub struct ApiOptions {
    /// Open store whose read metadata is already imported; the host closes it.
    pub store: Rc<Store>,
    /// Media handle with the `createMedia` operations.
    pub media: Rc<dyn Media>,
    /// Chunk storage for uploads.
    pub upload_storage: Rc<dyn UploadStorage>,
    /// Page screenshots for link capture.
    pub screenshot: Rc<dyn Screenshot>,
    /// Validated HTTP transport for link capture and remote attachments.
    pub resource: Rc<dyn Resource>,
    /// Outbound HTTP for the hosted AI and embedding APIs.
    pub fetch: Rc<dyn Fetch>,
    /// Acknowledge writes without parsing or persisting them.
    pub readonly: bool,
    /// Rewrites recorded origins to local paths.
    pub localize: fn(&str) -> String,
    /// Broadcasts a committed hub event: `(target, payload)`.
    pub push: Push,
    /// Whether the server can replay this exact request.
    pub has_replay: HasReplay,
    /// Random bytes behind share and upload ids.
    pub random: fn(usize) -> Vec<u8>,
}

/// The library API (`createApi`'s result).
pub struct Api {
    store: Rc<Store>,
    writer: Rc<Writer>,
    search: Rc<Search>,
    app: AppApi,
    media: Rc<dyn Media>,
    uploads: Rc<Uploads>,
    jobs: Enrichment,
    writes: MediaWrites,
    readonly: bool,
    localize: fn(&str) -> String,
    has_replay: HasReplay,
}

enum Mutation {
    Create,
    CreateSpace,
    ReorderSpaces,
    ReorderPins,
    Convert,
    Acknowledge,
    Serendipity,
    Dismiss(String),
    Batch(String),
    Patch(String),
    Trash(String, bool),
    Prose(String, String),
    Commit(String),
    AddTag(String),
    Flag(String, String),
    RemoveTag(String),
    PatchSpace(String),
    DeleteSpace(String),
    Membership(String, String, bool),
    Exclude(String, String),
    Settings(String),
    Onboard(String, bool),
    RestoreRevision(String, String),
    Share(String),
    RevokeShare(String, String),
    RemoveThumbnail(String),
}

fn parse(text: &str) -> Result<Value> {
    Ok(json::parse(text)?)
}

fn stringify(value: &Value) -> Result<String> {
    Ok(json::stringify(value)?.unwrap_or_default())
}

fn captures(pattern: &Regex, path: &str) -> Option<Vec<String>> {
    pattern.captures(path).map(|found| found.iter().map(|group| group.map_or(String::new(), |group| group.as_str().to_owned())).collect())
}

fn json_headers() -> Headers {
    Headers::from([("content-type", JSON_TYPE)])
}

fn type_header(content_type: &str) -> Headers {
    Headers::from([("content-type", content_type)])
}

// `body.key` as JS reads it, a TypeError on null.
fn member<'a>(body: &'a Value, key: &str) -> Result<&'a Value> {
    match body {
        Value::Undefined | Value::Null => Err(Error::invalid(format!("Cannot read properties of null (reading '{key}')"))),
        other => Ok(other.get(key)),
    }
}

fn to_int32(value: f64) -> i32 {
    if !value.is_finite() {
        return 0;
    }
    value.trunc().rem_euclid(4_294_967_296.0) as u32 as i32
}

// `Object.fromEntries(url.searchParams)`: the last value of a repeated name wins.
fn search_object(url: &Url) -> Value {
    let mut object = Object::new();
    for (name, value) in url.query_pairs() {
        object.insert(name.into_owned(), value.into_owned());
    }
    Value::Object(object)
}

fn timestamp_value(ns: i128) -> Value {
    Value::Ext(EXT_TIMESTAMP, timestamp((ns / 1_000_000_000) as f64, (ns % 1_000_000_000) as f64))
}

// Parse a bounded JSON request without including private request data in errors.
async fn request_body(request: &mut Request) -> Result<Value> {
    let bytes = request.take_body().bytes_within(BODY_LIMIT).await?;
    let text = String::from_utf8_lossy(&bytes).into_owned();
    if request.header("content-type").is_some_and(|kind| kind.starts_with("text/markdown")) {
        return Ok(Value::String(text));
    }
    if text.is_empty() {
        return Ok(obj! {}.into());
    }
    json::parse(&text).map_err(|_| Error::status(400, "Invalid JSON body"))
}

// The current card model used by both object reads and change messages.
fn object_model(row: &CardRow, space_ids: &[String]) -> Result<Value> {
    let data = parse(&row.data)?;
    let extras = parse(&row.extras)?;
    let spaces: Vec<Value> = space_ids.iter().map(|id| Value::Object(obj! { "id" => id.as_str() })).collect();
    let mut model = data.as_object().cloned().unwrap_or_default();
    model.insert("id", row.id.as_str());
    model.insert("entityType", data.get("type").clone());
    model.insert("spaces", Value::Array(spaces));
    let link = [extras.get("linkUrl"), data.get("source").get("url")].into_iter().find(|value| !value.is_nullish()).cloned();
    let mut entity = obj! {
        "@type" => data.get("type").clone(),
        "@id" => link.unwrap_or_else(|| Value::from(row.id.as_str())),
        "title" => if data.get("title").is_nullish() { Value::from("") } else { data.get("title").clone() },
    };
    if data.get("author").truthy() {
        entity.insert("author", data.get("author").clone());
    }
    if extras.get("cover").truthy() {
        entity.insert("cover", extras.get("cover").clone());
    }
    model.insert("mainEntity", entity);
    if !matches!(extras.get("pinPosition"), Value::Undefined) {
        model.insert("position", extras.get("pinPosition").clone());
    }
    Ok(Value::Object(model))
}

fn inline(value: &str) -> Value {
    let mut parts = Vec::new();
    let mut at = 0;
    for found in INLINE.find_iter(value) {
        parts.push(&value[at..found.start()]);
        parts.push(found.as_str());
        at = found.end();
    }
    parts.push(&value[at..]);
    Value::Array(
        parts
            .into_iter()
            .filter(|part| !part.is_empty())
            .map(|part| {
                if let Some(link) = LINK.captures(part) {
                    let marks = mm_js::arr![obj! { "type" => "link", "attrs" => obj! { "href" => &link[2] } }];
                    return obj! { "type" => "text", "text" => &link[1], "marks" => marks }.into();
                }
                let mark = if part.starts_with("**") {
                    Some("bold")
                } else if part.starts_with('`') {
                    Some("code")
                } else if part.starts_with('*') {
                    Some("italic")
                } else {
                    None
                };
                let Some(mark) = mark else { return obj! { "type" => "text", "text" => part }.into() };
                let trim = if mark == "bold" { 2 } else { 1 };
                let text = mm_js::text::slice(part, trim, Some(-trim));
                obj! { "type" => "text", "text" => text, "marks" => mm_js::arr![obj! { "type" => mark }] }.into()
            })
            .collect(),
    )
}

fn text_node(text: String) -> Value {
    obj! { "type" => "text", "text" => text }.into()
}

// Convert markdown blocks the local editor supports into prose without external services.
fn convert_markdown(text: &Value) -> Result<Value> {
    let Value::String(text) = text else { return Err(Error::invalid("text.split is not a function")) };
    let mut content: Vec<Value> = Vec::new();
    let mut code: Option<Vec<String>> = None;
    let lines: Vec<&str> = text.split('\n').map(|line| line.strip_suffix('\r').unwrap_or(line)).collect();
    for line in lines {
        if line.starts_with("```") {
            match code.take() {
                Some(block) => content.push(obj! { "type" => "codeBlock", "content" => mm_js::arr![text_node(block.join("\n"))] }.into()),
                None => code = Some(Vec::new()),
            }
        } else if let Some(block) = &mut code {
            block.push(line.to_owned());
        } else if HEADING.is_match(line) {
            let space = line.find(' ').unwrap_or(0);
            content.push(obj! { "type" => "heading", "attrs" => obj! { "level" => space as f64 }, "content" => inline(&line[space + 1..]) }.into());
        } else if let Some(prefix) = LIST_ITEM.find(line) {
            let kind = if line.starts_with(|c: char| c.is_ascii_digit()) { "orderedList" } else { "bulletList" };
            if content.last().is_none_or(|last| last.get("type").as_str() != Some(kind)) {
                content.push(obj! { "type" => kind, "content" => Value::Array(Vec::new()) }.into());
            }
            let paragraph = obj! { "type" => "paragraph", "content" => inline(&line[prefix.end()..]) };
            let item = obj! { "type" => "listItem", "content" => mm_js::arr![paragraph] };
            if let Some(Value::Array(items)) = content.last_mut().and_then(|last| last.get_mut("content")) {
                items.push(item.into());
            }
        } else if let Some(quote) = line.strip_prefix("> ") {
            let paragraph = obj! { "type" => "paragraph", "content" => inline(quote) };
            content.push(obj! { "type" => "blockquote", "content" => mm_js::arr![paragraph] }.into());
        } else if RULE.is_match(line) {
            content.push(obj! { "type" => "horizontalRule" }.into());
        } else if line.is_empty() {
            content.push(obj! { "type" => "paragraph" }.into());
        } else {
            content.push(obj! { "type" => "paragraph", "content" => inline(line) }.into());
        }
    }
    if let Some(block) = code {
        let inner = if block.is_empty() { Vec::new() } else { vec![text_node(block.join("\n"))] };
        content.push(obj! { "type" => "codeBlock", "content" => Value::Array(inner) }.into());
    }
    Ok(obj! { "type" => "doc", "content" => Value::Array(content) }.into())
}

// Replace the Top of Mind contents while respecting nested divs in card previews.
fn pinned_list(html: &str, contents: &str) -> String {
    let Some(start) = TOM_LIST.find(html) else { return html.to_owned() };
    let offset = start.end();
    let mut depth = 1;
    for tag in DIV_TAG.find_iter(&html[offset..]) {
        depth += if tag.as_str().starts_with("</") { -1 } else { 1 };
        if depth == 0 {
            return format!("{}{contents}{}", &html[..offset], &html[offset + tag.start()..]);
        }
    }
    html.to_owned()
}

fn pack(items: &[Value]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for item in items {
        encode_into(&mut out, item, EncodeOptions { use_big_int64: true })?;
    }
    Ok(out)
}

fn problem(error: Error) -> Result<Response> {
    let (status, detail) = match &error {
        Error::Status { status, message, .. } => (*status, message.clone()),
        Error::Invalid(_) => (400, "Invalid request data".to_owned()),
        Error::Internal(_) => return Err(error),
    };
    let body = stringify(&obj! { "type" => "about:blank", "status" => f64::from(status), "detail" => detail }.into())?;
    Ok(Response::send(status, body, &type_header("application/problem+json")))
}

/// Checked-in page shell template with recorded card data stripped.
pub const SHELL_TEMPLATE: &str = include_str!("../templates/shell.html");

impl Api {
    /// Open the API over a store (`createApi` with `store`): sweep stale uploads and requeue unfinished local cards
    /// unless read-only.
    ///
    /// # Arguments
    /// * `options` - Store, media, uploads, capture transports, policy and server callbacks; `OFFLINE=1` in the
    ///   environment limits capture and search to loopback services.
    ///
    /// # Errors
    /// A store without a user, or the uploads' or database's error.
    pub async fn new(options: ApiOptions) -> Result<Self> {
        let offline = runtime::env("OFFLINE").as_deref() == Some("1");
        let store = options.store;
        let writer = Rc::new(Writer::new(Rc::clone(&store), options.push)?.with_random(options.random));
        let search = Rc::new(Search::new(Rc::clone(&store), Some(Rc::clone(&options.fetch)), offline)?);
        let app = AppApi::new(Rc::clone(&store), Rc::clone(&search), options.localize);
        let uploads = Rc::new(Uploads::new(
            Rc::clone(&store),
            options.upload_storage,
            Rc::clone(&options.media),
            UploadOptions { random: options.random, ..UploadOptions::default() },
        )?);
        if !options.readonly {
            uploads.sweep().await?;
        }
        let jobs = Enrichment::new(EnrichmentOptions {
            store: Rc::clone(&store),
            writer: Rc::clone(&writer),
            media: Rc::clone(&options.media),
            resource: Rc::clone(&options.resource),
            screenshot: options.screenshot,
            fetch: options.fetch,
            offline,
            readonly: options.readonly,
        });
        if !options.readonly {
            let unfinished = store.db().query(
                "SELECT id, extras FROM cards WHERE json_extract(extras, '$.local') = 1 AND ((flags & 1) != 0 OR json_extract(extras, '$.capturePending') = 1)",
                &[],
            )?;
            for row in &unfinished {
                let extras = parse(row.text("extras").unwrap_or(""))?;
                jobs.schedule(row.text("id").unwrap_or(""), extras.get("capturePending").truthy());
            }
        }
        let writes = MediaWrites {
            store: Rc::clone(&store),
            writer: Rc::clone(&writer),
            media: Rc::clone(&options.media),
            uploads: Rc::clone(&uploads),
            jobs: jobs.clone(),
            resource: options.resource,
            offline,
        };
        Ok(Self {
            store,
            writer,
            search,
            app,
            media: options.media,
            uploads,
            jobs,
            writes,
            readonly: options.readonly,
            localize: options.localize,
            has_replay: options.has_replay,
        })
    }

    /// The enrichment queue, for tests and shutdown to drain.
    pub fn jobs(&self) -> &Enrichment {
        &self.jobs
    }

    /// Stop the enrichment queue (`close`); the host closes the store.
    pub fn close(&self) {
        self.jobs.close();
    }

    fn localize(&self, text: &str) -> String {
        (self.localize)(text)
    }

    fn quota(&self) -> Result<Headers> {
        Ok(Headers::from([("x-object-quota", stringify(&self.store.quota(None)?.to_value())?)]))
    }

    fn with_quota(&self, headers: Headers) -> Result<Headers> {
        Ok(headers.with(&self.quota()?))
    }

    fn require_card(&self, id: &str) -> Result<CardRow> {
        self.store.card(id, None)?.ok_or_else(|| Error::status(404, "Card not found"))
    }

    fn card_value(&self, row: &CardRow) -> Value {
        obj! { "html" => self.localize(&row.html), "json" => self.localize(&row.data) }.into()
    }

    // mymind sends an iPhone smaller cover renditions; other devices get the desktop card HTML.
    fn device_card(&self, row: &CardRow, variant: &str) -> Result<Value> {
        let mut card = self.card_value(row);
        if variant == "iphone" {
            let html = iphone_card(card.get("html").as_str().unwrap_or(""), card.get("json").as_str().unwrap_or(""))?;
            card.set("html", html);
        }
        Ok(card)
    }

    fn device_cards(&self, rows: &[CardRow], variant: &str) -> Result<Vec<u8>> {
        pack(&rows.iter().map(|row| self.device_card(row, variant)).collect::<Result<Vec<_>>>()?)
    }

    // Read share timestamps as text so SQLite never rounds or rejects nanoseconds.
    fn active_share(&self, id: &str, by_card: bool) -> Result<Option<crate::sql::Row>> {
        let column = if by_card { "card_id" } else { "id" };
        let now = SqlValue::Integer(date::now() as i64 * 1_000_000);
        self.store.db().one(
            &format!(
                "SELECT id, card_id, CAST(created_ns AS TEXT) AS created_ns, CAST(expires_ns AS TEXT) AS expires_ns
      FROM shares WHERE {column} = ? AND expires_ns > ? ORDER BY shares.created_ns DESC LIMIT 1"
            ),
            &[id.into(), now],
        )
    }

    // Encode a poll item with exact uint64 checkpoints and MessagePack timestamp dates, in the device's card HTML.
    fn change_value(&self, change: &Change, variant: &str, spaces: &HashMap<String, Vec<String>>) -> Result<Value> {
        let mut object = if change.object_type == 190 {
            match self.store.space(&change.object_id)? {
                Some(space) => space.to_value(),
                None => obj! { "id" => change.object_id.as_str() }.into(),
            }
        } else if let Some(card) = &change.card {
            object_model(card, spaces.get(&card.id).map_or(&[], Vec::as_slice))?
        } else {
            obj! { "id" => change.object_id.as_str() }.into()
        };
        for key in ["created", "bumped", "modified", "deleted"] {
            if object.get(key).truthy() {
                let iso = object.get(key).to_js_string();
                object.set(key, timestamp_value(ns_from_iso(&iso)?));
            }
        }
        let hidden_draft = change.card.as_ref().is_some_and(|card| card.flags & 131_072 != 0);
        let (card_html, card_data) = match &change.card {
            None => (String::new(), String::new()),
            Some(card) if variant == "iphone" => (iphone_card(&self.localize(&card.html), &self.localize(&card.data))?, self.localize(&card.data)),
            Some(card) => (self.localize(&card.html), self.localize(&card.data)),
        };
        Ok(obj! {
            "type" => if hidden_draft { 2.0 } else { change.kind as f64 },
            "timestamp" => Value::BigInt(change.ts),
            "objectType" => change.object_type as f64,
            "object" => object,
            "cardHtml" => card_html,
            "cardData" => card_data,
        }
        .into())
    }

    // Restore recorded transport headers while generating current bodies and quota headers.
    fn reply(&self, url: &Url, body: impl Into<crate::http::Body>, status: u16, headers: Headers) -> Result<Response> {
        let key = format!("{}{}", url.path(), crate::http::search(url));
        let mut all = Headers::new();
        if let Some(saved) = self.store.db().one("SELECT headers FROM api_metadata WHERE path = ?", &[key.into()])?
            && let Value::Object(recorded) = parse(saved.text("headers").unwrap_or(""))?
        {
            for (name, value) in recorded.iter() {
                all.set(&*name, value.to_js_string());
            }
        }
        Ok(Response::send(status, body, &all.with(&headers)))
    }

    fn reply_json(&self, url: &Url, body: &Value) -> Result<Response> {
        self.reply(url, stringify(body)?, 200, json_headers())
    }

    // The route a write matches, without reading the request body.
    fn mutation(&self, method: &str, path: &str) -> Result<Option<Mutation>> {
        let post = method == "POST";
        let found = |pattern: &Regex| captures(pattern, path);
        Ok(Some(match () {
            () if post && path == "/objects" => Mutation::Create,
            () if post && path == "/spaces" => Mutation::CreateSpace,
            () if post && path == "/spaces/reorder" => Mutation::ReorderSpaces,
            () if post && path == "/objects/reorder-pins" => Mutation::ReorderPins,
            () if post && path == "/convert" => Mutation::Convert,
            () if post && (path == "/jserrors" || path == "/browser-extension/activate") => Mutation::Acknowledge,
            () if post && path == "/serendipity/complete" => Mutation::Serendipity,
            () => {
                if post && let Some(m) = found(&DISMISS) {
                    return Ok(Some(Mutation::Dismiss(m[1].clone())));
                }
                if post && let Some(m) = found(&BATCH) {
                    return Ok(Some(Mutation::Batch(m[1].clone())));
                }
                if let Some(m) = found(&OBJECT) {
                    match method {
                        "PATCH" => return Ok(Some(Mutation::Patch(m[1].clone()))),
                        "DELETE" => return Ok(Some(Mutation::Trash(m[1].clone(), false))),
                        _ => {}
                    }
                }
                if let Some(m) = found(&OBJECT_ACTION) {
                    let (id, action) = (m[1].clone(), m[2].as_str());
                    if (method == "PUT" && ["prose", "note", "content"].contains(&action)) || (method == "PATCH" && action == "prose") {
                        return Ok(Some(Mutation::Prose(id, action.to_owned())));
                    }
                    if method == "PATCH" && action == "entity" {
                        return Ok(Some(Mutation::Patch(id)));
                    }
                    if method == "PATCH" && action == "commit" {
                        return Ok(Some(Mutation::Commit(id)));
                    }
                    if post {
                        match action {
                            "tags" => return Ok(Some(Mutation::AddTag(id))),
                            "commit" => return Ok(Some(Mutation::Commit(id))),
                            "restore" => return Ok(Some(Mutation::Trash(id, true))),
                            "pin" | "unpin" | "complete" | "uncomplete" => return Ok(Some(Mutation::Flag(id, action.to_owned()))),
                            _ => {}
                        }
                    }
                }
                if method == "DELETE" && let Some(m) = found(&ANNOTATION) {
                    return Ok(Some(Mutation::RemoveTag(m[1].clone())));
                }
                if let Some(m) = found(&SPACE) {
                    match method {
                        "PATCH" => return Ok(Some(Mutation::PatchSpace(m[1].clone()))),
                        "DELETE" => return Ok(Some(Mutation::DeleteSpace(m[1].clone()))),
                        _ => {}
                    }
                }
                if ["PUT", "DELETE"].contains(&method) && let Some(m) = found(&MEMBERSHIP) {
                    return Ok(Some(Mutation::Membership(m[1].clone(), m[2].clone(), method == "DELETE")));
                }
                if post && let Some(m) = found(&EXCLUDE) {
                    return Ok(Some(Mutation::Exclude(m[1].clone(), m[2].clone())));
                }
                if method == "PATCH" && let Some(m) = found(&USER) {
                    return Ok(Some(Mutation::Settings(m[1].clone())));
                }
                if post && let Some(m) = found(&ONBOARD) {
                    return Ok(Some(Mutation::Onboard(m[1].clone(), m[2] == "set")));
                }
                if post && let Some(m) = found(&REVISION_RESTORE) {
                    return Ok(Some(Mutation::RestoreRevision(m[1].clone(), m[2].clone())));
                }
                if post && let Some(m) = found(&SHARES) {
                    return Ok(Some(Mutation::Share(m[1].clone())));
                }
                if method == "DELETE" && let Some(m) = found(&SHARE) {
                    return Ok(Some(Mutation::RevokeShare(m[1].clone(), m[2].clone())));
                }
                if post && let Some(m) = found(&THUMBNAIL) {
                    return Ok(Some(Mutation::RemoveThumbnail(m[1].clone())));
                }
                return Ok(None);
            }
        }))
    }

    // One write inside the writer's transaction: `{ status?, body?, headers? }` as the JS route returned it.
    fn apply(&self, writer: &Writer, mutation: &Mutation, body: &Value) -> Result<Value> {
        let wrap = |value: Value| -> Result<Value> { Ok(obj! { "body" => value }.into()) };
        match mutation {
            Mutation::Create => writer.create(body, &Value::Undefined),
            Mutation::CreateSpace => wrap(writer.create_space(body)?),
            Mutation::ReorderSpaces => wrap(writer.reorder_spaces(body)?),
            Mutation::ReorderPins => wrap(writer.reorder_pins(body)?),
            Mutation::Convert => Ok(obj! { "body" => convert_markdown(body)?, "headers" => obj! { "content-type" => "application/prose+json" } }.into()),
            Mutation::Acknowledge => wrap(obj! {}.into()),
            Mutation::Serendipity => wrap(writer.remember("completedFlows", &"serendipity".into())?),
            Mutation::Dismiss(id) => wrap(writer.remember("dismissedPromotions", &id.as_str().into())?),
            Mutation::Batch(action) => {
                let Value::Array(ids) = member(body, "objectIds")? else { return Err(Error::status(400, "Expected objectIds array")) };
                for id in ids {
                    match action.as_str() {
                        "tag" => {
                            writer.add_tag(&bind_id(id)?, body.get("tag"))?;
                        }
                        "delete" => {
                            writer.trash(&bind_id(id)?, false)?;
                        }
                        _ => {
                            writer.membership(&bind_id(body.get("spaceId"))?, &bind_id(id)?, false)?;
                        }
                    }
                }
                wrap(obj! {}.into())
            }
            Mutation::Patch(id) => wrap(writer.patch(id, body)?),
            Mutation::Trash(id, restore) => wrap(writer.trash(id, *restore)?),
            Mutation::Prose(id, action) => wrap(writer.prose(id, body, action)?),
            Mutation::Commit(id) => writer.commit(id, member(body, "prose")?),
            Mutation::AddTag(id) => wrap(writer.add_tag(id, member(body, "name")?)?),
            Mutation::Flag(id, action) => {
                let bit = if action.contains("pin") { 512 } else { 65_536 };
                wrap(writer.flag(id, bit, action == "pin" || action == "complete", member(body, "position")?)?)
            }
            Mutation::RemoveTag(id) => wrap(writer.remove_tag(id)?),
            Mutation::PatchSpace(id) => wrap(writer.patch_space(id, body)?),
            Mutation::DeleteSpace(id) => wrap(writer.delete_space(id)?),
            Mutation::Membership(space, id, remove) => wrap(writer.membership(space, id, *remove)?),
            Mutation::Exclude(query, id) => wrap(writer.exclude(query, id)?),
            Mutation::Settings(id) => {
                let user = self.store.user(None)?.ok_or_else(|| Error::invalid("Cannot read properties of undefined (reading 'id')"))?;
                if *id != user.id && *id != user.account_uid {
                    return Err(Error::status(404, "User not found"));
                }
                wrap(writer.settings(body)?)
            }
            Mutation::Onboard(value, set) => wrap(writer.onboard(to_int32(number::string_to_number(value)), *set)?),
            Mutation::RestoreRevision(id, revision) => wrap(writer.restore_revision(id, revision)?),
            Mutation::Share(id) => wrap(writer.share(id)?),
            Mutation::RevokeShare(id, share) => wrap(writer.revoke_share(id, share)?),
            Mutation::RemoveThumbnail(id) => wrap(writer.remove_thumbnail(id)?),
        }
    }

    async fn write(&self, request: &mut Request, method: &str, path: &str) -> Result<Option<Response>> {
        let Some(mutation) = self.mutation(method, path)? else { return Ok(None) };
        let mut body = request_body(request).await?;
        let creates = matches!(mutation, Mutation::Create);
        if creates {
            extension_body(&mut body);
        }
        let upload = if creates && (member(&body, "uploadId")?.truthy() || body.get("type").as_str() == Some("Media")) {
            let key = upload_id(&self.uploads, body.get("uploadId")).await?;
            let prepared = self.uploads.prepare(&key).await?;
            Some((key, prepared))
        } else {
            None
        };
        let result = self.writer.run(|writer| {
            let Some((key, prepared)) = &upload else { return self.apply(writer, &mutation, &body) };
            if prepared.get("existingId").truthy() {
                return Ok(obj! { "status" => 200.0, "body" => obj! { "id" => prepared.get("existingId").clone() } }.into());
            }
            let result = writer.create(&body, prepared)?;
            self.uploads.claim(key, &result.get("body").get("id").to_js_string(), false)?;
            Ok(result)
        })?;
        if let Some((key, _)) = &upload {
            self.uploads.cleanup_claimed(key).await?;
        }
        let status = if result.get("status").is_nullish() { 200 } else { result.get("status").to_number() as u16 };
        let sent = if result.get("body").is_nullish() { obj! {}.into() } else { result.get("body").clone() };
        let mut headers = self.with_quota(json_headers())?;
        if let Value::Object(extra) = result.get("headers") {
            for (name, value) in extra.iter() {
                headers.set(&*name, value.to_js_string());
            }
        }
        let response = Response::send(status, stringify(&sent)?, &headers);
        // The JS route had already sent its answer here, so a failure below could not change it.
        let _ = self.schedule_after(method, path, &mutation, status, &result);
        Ok(Some(response))
    }

    fn schedule_after(&self, method: &str, path: &str, mutation: &Mutation, status: u16, result: &Value) -> Result<()> {
        let schedule_row = |row: Option<CardRow>| -> Result<()> {
            let row = row.ok_or_else(|| Error::invalid("Cannot read properties of undefined (reading 'id')"))?;
            let capture = parse(&row.extras)?.get("capturePending").truthy();
            self.jobs.schedule(&row.id, capture);
            Ok(())
        };
        if matches!(mutation, Mutation::Create) && status == 201 {
            return schedule_row(self.store.card(&result.get("body").get("id").to_js_string(), None)?);
        }
        if method == "POST" && let Some(m) = captures(&RESTORE, path) {
            return schedule_row(self.store.card(&m[1], None)?);
        }
        if ["PUT", "PATCH", "POST"].contains(&method) && let Some(m) = captures(&PROSE_WRITE, path)
            && let Some(row) = self.store.card(&m[1], None)?
            && ["Note", "Content", "Article"].contains(&row.kind.as_str())
        {
            self.jobs.schedule(&row.id, false);
        }
        Ok(())
    }

    async fn route(&self, request: &mut Request, url: &Url, path: &str) -> Result<Option<Response>> {
        let method = request.method.clone();
        let method = method.as_str();
        let variant = device_variant(request.header("user-agent").unwrap_or(""));
        let phone = !variant.is_empty();
        let get = method == "GET";
        if let Some(response) = self.app.handle(method, url).await? {
            return Ok(Some(response));
        }
        if (get || method == "HEAD") && let Some(relative) = path.strip_prefix("/media/") {
            let Some(result) = self.media.response(relative).await? else {
                return Ok(Some(Response::send(404, "", &type_header("text/plain"))));
            };
            return Ok(Some(self.media.send(&result, method, request.header("range")).await?));
        }
        if !self.readonly && method == "POST" && path == "/uploads/authorize" {
            let count = search_param(url, "count").map_or(Value::Null, Value::String);
            let body = stringify(&self.uploads.authorize(&count).await?)?;
            return Ok(Some(Response::send(200, body, &self.with_quota(json_headers())?)));
        }
        if !self.readonly && let Some((status, body)) = self.writes.handle(request).await? {
            return Ok(Some(Response::send(status, stringify(&body)?, &self.with_quota(json_headers())?)));
        }
        if !self.readonly && method == "PUT" && let Some(m) = captures(&UPLOAD_PUT, path) {
            let upload = UploadRequest { headers: request.headers.clone(), body: request.take_body() };
            let body = stringify(&self.uploads.put(&m[1], upload).await?)?;
            return Ok(Some(Response::send(200, body, &self.with_quota(json_headers())?)));
        }
        if !self.readonly && method == "POST" && let Some(m) = captures(&ANALYZE, path) {
            let row = self.require_card(&m[1])?;
            let data = parse(&row.data)?;
            self.jobs.schedule(&row.id, false);
            let body = if self.jobs.enabled() {
                obj! { "status" => "Processing" }.into()
            } else {
                let summary = if data.get("summary").is_nullish() { Value::from("") } else { data.get("summary").clone() };
                let tags: Vec<Value> = self.store.card_tags(&row.id)?.iter().map(|tag| tag.to_value()).collect();
                obj! { "status" => "Skipped", "summary" => summary, "tags" => Value::Array(tags) }.into()
            };
            return Ok(Some(Response::send(200, stringify(&body)?, &self.with_quota(json_headers())?)));
        }
        if get && search_param(url, "dismiss").as_deref() == Some("true") && let Some(m) = captures(&ACHIEVEMENT, path) {
            let body = if self.readonly { obj! {}.into() } else { self.writer.run(|writer| writer.remember("dismissedAchievements", &m[1].as_str().into()))? };
            return Ok(Some(self.reply_json(url, &body)?));
        }
        if !phone && get && let Some(m) = captures(&SHARE_PAGE, path) {
            let card = match self.active_share(&m[1], false)? {
                Some(share) => self.store.card(share.text("card_id").unwrap_or(""), None)?.filter(|card| card.deleted_ns.is_none()),
                None => None,
            };
            return Ok(Some(match card {
                None => Response::send(410, "Share expired or revoked", &type_header("text/plain; charset=utf-8")),
                Some(card) => Response::send(200, self.localize(&render_share(&card)?), &type_header(HTML_TYPE)),
            }));
        }
        let page_load = request.header("sec-fetch-dest") == Some("document")
            || (request.header("x-partial").is_none()
                && (matches!(path, "/" | "/everything" | "/trash" | "/account" | "/achievements" | "/bookshelf")
                    || path.starts_with("/things/")
                    || (request.header("accept").is_some_and(|accept| accept.contains("text/html"))
                        && (path == "/spaces" || path.starts_with("/spaces/") || path == "/serendipity"))));
        if get && page_load {
            return Ok(Some(Response::send(200, self.shell()?, &type_header(HTML_TYPE))));
        }
        if get && (["/search", "/search/suggestions"].contains(&path) || QUERY_CHECK.is_match(path)) && self.readonly && (self.has_replay)(request)? {
            return Ok(None);
        }
        let response = if get && path == "/search" {
            let mut options = search_object(url);
            let q = options.get("q").as_str().unwrap_or("").to_owned();
            options.set("wildcard", options.get("wildcard").as_str() == Some("true"));
            options.set("semantic", options.get("semantic").as_str() == Some("true"));
            self.reply_json(url, &self.search.search(&q, &options).await?.to_value())?
        } else if get && path == "/search/suggestions" {
            let found = self.search.suggestions(&search_param(url, "q").unwrap_or_default())?;
            self.reply_json(url, &Value::Array(found.iter().map(|suggestion| suggestion.to_value()).collect()))?
        } else if get && path == "/embeddings/search" {
            let found = self.search.embeddings(&search_param(url, "q").unwrap_or_default(), &search_object(url)).await?;
            self.reply_json(url, &found.to_value())?
        } else if get && let Some(m) = captures(&RELATED, path) {
            self.reply_json(url, &self.search.related(&m[1], &search_object(url))?.to_value())?
        } else if get && let Some(m) = captures(&QUERY_CHECK, path) {
            let check = if self.readonly { self.writer.check_query(&m[1], false)? } else { self.writer.run(|writer| writer.check_query(&m[1], true))? };
            self.reply_json(url, &check.to_value())?
        } else if method == "POST" && path == "/cards/grid" {
            let Value::Array(ids) = request_body(request).await? else { return Err(Error::status(400, "Expected card id array")) };
            let ids = ids.iter().map(bind_id).collect::<Result<Vec<_>>>()?;
            let rows = self.store.cards_by_ids(&ids, None)?;
            self.reply(url, self.device_cards(&rows, variant)?, 200, type_header(MSGPACK))?
        } else if get && (path == "/cards" || path == "/cards/trash") {
            let grid = path == "/cards";
            let rows = if grid { self.store.grid_cards(None, 0, None)? } else { self.store.trash_cards(None)? };
            let mut headers = type_header(MSGPACK);
            if grid {
                headers = self.with_quota(headers)?;
                headers.set("x-checkpoint", self.store.grid_checkpoint(None)?);
                headers.set("access-control-expose-headers", "x-object-quota, x-checkpoint");
            }
            self.reply(url, self.device_cards(&rows, variant)?, 200, headers)?
        } else if get && let Some(m) = captures(&MARKDOWN, path) {
            let row = self.require_card(&m[1])?;
            let content = match self.store.db().one("SELECT body FROM card_content WHERE card_id = ?", &[row.id.as_str().into()])? {
                Some(found) => parse(found.text("body").unwrap_or(""))?,
                None => Value::Null,
            };
            self.reply(url, self.localize(&card_markdown(&parse(&row.data)?, &content)), 200, type_header("text/markdown; charset=utf-8"))?
        } else if get && let Some(m) = captures(&DETAIL, path) {
            let row = self.require_card(&m[1])?;
            let share = match self.active_share(&row.id, true)? {
                Some(share) => {
                    let id = share.text("id").unwrap_or("").to_owned();
                    let expires = number::string_to_number(share.text("expires_ns").unwrap_or("")) / 1_000_000.0;
                    let expires = date::iso_from_ms(expires).ok_or_else(|| Error::invalid("Invalid time value"))?;
                    obj! { "id" => id.as_str(), "url" => format!("/shares/{id}"), "expires" => expires }.into()
                }
                None => Value::Null,
            };
            let content = match self.store.db().one("SELECT body FROM card_content WHERE card_id = ?", &[row.id.as_str().into()])? {
                Some(found) => parse(found.text("body").unwrap_or(""))?,
                None => Value::Null,
            };
            let headers = Headers::from([("content-type", HTML_TYPE), ("x-partial", "CardDetail")]);
            self.reply(url, self.localize(&render_detail(&row, &share, &content)?), 200, headers)?
        } else if get && search_param(url, "template").as_deref() == Some("PinnedCard") && let Some(m) = captures(&CARD, path) {
            let row = self.require_card(&m[1])?;
            self.reply(url, self.localize(&render_pinned(&row)), 200, Headers::from([("content-type", HTML_TYPE), ("x-partial", "PinnedCard")]))?
        } else if get && !url.query_pairs().any(|(name, _)| name == "template") && let Some(m) = captures(&CARD, path) {
            let row = self.require_card(&m[1])?;
            if request.header("accept").is_some_and(|accept| accept.contains("application/json")) {
                self.reply(url, self.localize(&row.data), 200, json_headers())?
            } else {
                self.reply(url, pack(&[self.device_card(&row, variant)?])?, 200, type_header(MSGPACK))?
            }
        } else if get && path == "/spaces" {
            self.reply_json(url, &Value::Array(self.store.spaces(None)?.iter().map(|space| space.to_value()).collect()))?
        } else if get && let Some(m) = captures(&SPACE, path) {
            let mut space = self.store.space(&m[1])?.ok_or_else(|| Error::status(404, "Space not found"))?.to_value();
            let metadata = self.store.db().one("SELECT attributes FROM api_metadata WHERE path = ?", &[path.into()])?;
            if space.get("query").truthy()
                && let Some(metadata) = metadata
            {
                let attributes = parse(metadata.text("attributes").unwrap_or(""))?;
                let checked = member(&attributes, "checked")?.clone();
                if let Some(query) = space.get_mut("query")
                    && !checked.is_nullish()
                {
                    query.set("checked", checked);
                }
            }
            self.reply_json(url, &space)?
        } else if get && path == "/tags/manual" {
            let tags: Vec<Value> = self.store.tags_manual(None)?.iter().map(|tag| tag.to_value()).collect();
            if request.header("accept").is_some_and(|accept| accept.contains("application/json") && !accept.contains(MSGPACK)) {
                self.reply_json(url, &Value::Array(tags))?
            } else {
                self.reply(url, pack(&[Value::Array(tags)])?, 200, type_header(MSGPACK))?
            }
        } else if get && path == "/tags/recent" {
            let take = search_param(url, "take").map_or(10.0, |take| number::string_to_number(&take));
            let names = self.store.tags_recent(take as i64, None)?;
            self.reply_json(url, &Value::Array(names.into_iter().map(|name| obj! { "name" => name }.into()).collect()))?
        } else if get && path == "/changes" {
            let since = search_param(url, "since").unwrap_or_else(|| "0".to_owned());
            if !DIGITS.is_match(&since) {
                return Err(Error::status(400, "Invalid checkpoint"));
            }
            // The checkpoint binds as text, so one past every stored nanosecond count matches nothing either way.
            let since = since.parse::<i128>().unwrap_or(i128::MAX);
            let changes = self.store.changes_since(since, &ChangesOptions { coalesce: true, ..ChangesOptions::default() })?;
            let card_ids: Vec<&str> = changes.iter().filter_map(|change| change.card.as_ref().map(|card| card.id.as_str())).collect();
            let spaces = self.store.card_spaces(&card_ids)?;
            let items = changes.iter().map(|change| self.change_value(change, variant, &spaces)).collect::<Result<Vec<_>>>()?;
            let mut headers = self.with_quota(type_header(MSGPACK))?;
            headers.set("x-count", changes.len().to_string());
            headers.set("access-control-expose-headers", "x-object-quota, x-count");
            self.reply(url, pack(&items)?, 200, headers)?
        } else if get && let Some(m) = captures(&OBJECT_READ, path) {
            let row = self.require_card(&m[1])?;
            let data = parse(&row.data)?;
            let spaces = self.store.card_spaces(&[row.id.as_str()])?.remove(&row.id).unwrap_or_default();
            let mut body = object_model(&row, &spaces)?;
            let mut headers = json_headers();
            match m[2].as_str() {
                "tags" => body = Value::Array(self.store.card_tags(&row.id)?.iter().map(|tag| tag.to_value()).collect()),
                "content" => {
                    let content = self.store.db().one("SELECT body FROM card_content WHERE card_id = ?", &[row.id.as_str().into()])?;
                    if content.is_none() && !data.get("prose").truthy() {
                        return Ok(None);
                    }
                    body = match content {
                        Some(found) => parse(found.text("body").unwrap_or(""))?,
                        None => data.get("prose").clone(),
                    };
                    headers = type_header("application/prose+json");
                }
                "revisions" => {
                    let revisions = self.store.db().query("SELECT * FROM revisions WHERE card_id = ? ORDER BY created DESC", &[row.id.as_str().into()])?;
                    if revisions.is_empty() {
                        return Ok(None);
                    }
                    let mut list = Vec::with_capacity(revisions.len());
                    for (index, revision) in revisions.iter().enumerate() {
                        let record = revision.to_object();
                        list.push(
                            obj! {
                                "id" => record.get("id").cloned().unwrap_or_default(),
                                "created" => record.get("created").cloned().unwrap_or_default(),
                                "isCurrent" => index == 0,
                                "content" => obj! { "body" => parse(revision.text("body").unwrap_or(""))? },
                            }
                            .into(),
                        );
                    }
                    body = Value::Array(list);
                }
                _ => {}
            }
            self.reply(url, self.localize(&stringify(&body)?), 200, headers)?
        } else if get && (path == "/tags/suggest" || path == "/tags/detail") {
            let detail = path == "/tags/detail";
            let name = search_param(url, if detail { "name" } else { "q" }).unwrap_or_default();
            let user = self.store.default_user_id()?;
            let body = if detail {
                let rows = self.store.db().query(
                    "SELECT DISTINCT a.card_id AS id, 1 AS score FROM annotations a
                JOIN cards c ON c.id = a.card_id WHERE a.user_id = ? AND a.name = ? AND c.deleted_ns IS NULL",
                    &[user.into(), name.into()],
                )?;
                obj! { "objects" => Value::Array(rows.iter().map(|row| Value::Object(row.to_object())).collect()) }.into()
            } else {
                let rows = self.store.db().query(
                    "SELECT name, COUNT(*) AS count FROM annotations WHERE user_id = ? AND instr(lower(name), lower(?)) > 0
                GROUP BY name ORDER BY count DESC, name LIMIT 20",
                    &[user.into(), name.into()],
                )?;
                Value::Array(rows.iter().map(|row| Value::Object(row.to_object())).collect())
            };
            self.reply_json(url, &body)?
        } else if get && (path == "/quota" || path == "/quotas/objects") {
            self.reply(url, stringify(&self.store.quota(None)?.to_value())?, 200, self.with_quota(json_headers())?)?
        } else if get && path == "/account.json" {
            let metadata = self.store.db().one("SELECT attributes FROM api_metadata WHERE path = ?", &[path.into()])?;
            let mut account = match metadata {
                Some(metadata) => parse(metadata.text("attributes").unwrap_or(""))?.get("account").clone(),
                None => obj! {}.into(),
            };
            if account.is_nullish() {
                return Err(Error::invalid("Cannot set properties of undefined (setting 'quotas')"));
            }
            let mut quotas = match account.get("quotas") {
                Value::Object(existing) => existing.clone(),
                _ => Object::new(),
            };
            quotas.insert("objects", self.store.quota(None)?.to_value());
            account.set("quotas", quotas);
            self.reply(url, stringify(&account)?, 200, self.with_quota(json_headers())?)?
        } else {
            if get || self.readonly {
                return Ok(None);
            }
            return self.write(request, method, path).await;
        };
        Ok(Some(response))
    }
}

#[async_trait(?Send)]
impl LibraryApi for Api {
    async fn handle(&self, request: &mut Request) -> Result<Option<Response>> {
        let url = request.url.clone();
        let path = uri::decode_uri(url.path())?;
        match self.route(request, &url, &path).await {
            Err(error) => problem(error).map(Some),
            other => other,
        }
    }

    fn shell(&self) -> Result<String> {
        let store = &self.store;
        let user = store.user(None)?.ok_or_else(|| Error::invalid("Cannot read properties of undefined (reading 'settings')"))?;
        let mut contents = String::new();
        for card in store.pinned_cards(None)? {
            contents.push_str(&self.localize(&render_pinned(&card)));
        }
        contents.push_str("\n            <div class=\"tom__empty \">\n              <p><i></i> Drop card here to keep it top of mind.</p>\n            </div>\n          ");
        let mut html = pinned_list(SHELL_TEMPLATE, &contents);
        // Onboarding flag 128 is the one the client sets after showing the Top of Mind introduction.
        let onboard = user.settings.get("onboardFlags");
        if to_int32(if onboard.is_nullish() { 0.0 } else { onboard.to_number() }) & 128 != 0 {
            html = html.replacen("<mymind-top-of-mind class=\"tom \">", "<mymind-top-of-mind class=\"tom introduced\">", 1);
        }
        let Some(found) = CONFIG.captures(&html) else { return Ok(self.localize(&html)) };
        let mut config = parse(&found[2])?;
        for key in ["colorScheme", "locale", "timeZoneId", "soundEnabled", "onboardFlags"] {
            if let Some(value) = user.settings.as_object().and_then(|settings| settings.get(key)) {
                config.set(key, value.clone());
            }
        }
        let flags = Value::Number(user.flags as f64);
        match config.get_mut("user") {
            Some(target @ Value::Object(_)) => {
                target.set("uid", Value::from(user.id.as_str()));
                target.set("flags", flags);
            }
            _ => return Err(Error::invalid("Cannot set properties of undefined (setting 'flags')")),
        }
        let quota = store.quota(None)?.to_value();
        match config.get_mut("quotas") {
            Some(target @ Value::Object(_)) => target.set("objects", quota),
            _ => return Err(Error::invalid("Cannot set properties of undefined (setting 'objects')")),
        }
        let replaced = format!("{}{}{}", &found[1], stringify(&config)?.replace('<', "\\u003c"), &found[3]);
        let range = found.get(0).expect("whole match").range();
        let mut out = format!("{}{replaced}{}", &html[..range.start], &html[range.end..]);
        match config.get("colorScheme").as_str() {
            Some("system") => {
                out = out.replacen(
                    "id=\"dark-stylesheet\" href=\"/_static/134/0.109.0/styles/dark.css\" media=\"all\"",
                    "id=\"dark-stylesheet\" href=\"/_static/134/0.109.0/styles/dark.css\" media=\"(prefers-color-scheme: dark)\"",
                    1,
                );
            }
            Some("light") => {
                out = out.replacen(
                    "id=\"dark-stylesheet\" href=\"/_static/134/0.109.0/styles/dark.css\" media=\"all\"",
                    "id=\"dark-stylesheet\" href=\"/_static/134/0.109.0/styles/dark.css\" media=\"not all\"",
                    1,
                );
            }
            _ => {}
        }
        Ok(self.localize(&out))
    }
}

/// Maps the browser extension's `WebPage` and URL-only `Media` bodies onto a link card, which the capture jobs fill in.
fn extension_body(body: &mut Value) {
    let kind = body.get("type").as_str().map(str::to_owned);
    let link = body.get("url").as_str().map(str::to_owned);
    let (Some(kind), Some(link)) = (kind, link) else { return };
    let plain_media = kind == "Media" && !body.get("uploadId").truthy();
    if (kind == "WebPage" || plain_media) && body.is_object() {
        body.set("content", link);
        body.set("type", "Link");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_converts_blocks_and_marks() {
        let doc = convert_markdown(&"# Hi **there**\n- a\n- [b](u)\n1. c\n> q\n---\n\n```\nx\n".into()).unwrap();
        let text = stringify(&doc).unwrap();
        assert!(text.contains(r#""level":1"#) && text.contains("bulletList") && text.contains("orderedList") && text.contains("codeBlock"));
    }

    #[test]
    fn pinned_list_keeps_nested_divs() {
        let html = r#"<div class="tom__list x"><div>a</div></div><p>"#;
        assert_eq!(pinned_list(html, "N"), r#"<div class="tom__list x">N</div><p>"#);
        assert_eq!(pinned_list(r#"<div class="tom__list_x">"#, "N"), r#"<div class="tom__list_x">"#);
    }
}
