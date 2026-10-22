//! First sign-in creates one empty library and reuses it. A second library cannot read the first library's card.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

use async_trait::async_trait;
use futures_executor::block_on;
use mm_core::api::{Api, ApiOptions};
use mm_core::auth_store::{self, Identity};
use mm_core::host::{
    AnalysisImage, Fetch, FetchedResource, Media, MediaInput, MediaResult, MediaUser,
    PreparedMedia, Resource, ResourceOptions, Screenshot, UploadStorage,
};
use mm_core::http::{Body, Request, Response};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::site::LibraryApi;
use mm_core::sql::{Sql, SqlExt};
use mm_core::store::Store;
use mm_core::store_member::{self, MemberProfile};
use mm_core::{Result, Value, obj};
use mm_js::json;
use mm_sqlite::Sqlite;

struct TestRuntime;
impl Runtime for TestRuntime {
    fn spawn(&self, _task: Task) {}
    fn sleep(&self, _ms: u64) -> Task {
        Box::pin(std::future::pending())
    }
    fn log(&self, _line: &str) {}
}

#[derive(Default)]
struct Memory(RefCell<HashMap<String, Vec<u8>>>);
#[async_trait(?Send)]
impl UploadStorage for Memory {
    fn key(&self) -> String {
        format!("library-claim-{:p}", self)
    }
    async fn names(&self) -> Result<Vec<String>> {
        Ok(self.0.borrow().keys().cloned().collect())
    }
    async fn size(&self, id: &str) -> Result<Option<u64>> {
        Ok(self.0.borrow().get(id).map(|bytes| bytes.len() as u64))
    }
    async fn truncate(&self, id: &str, size: u64) -> Result<()> {
        if let Some(bytes) = self.0.borrow_mut().get_mut(id) {
            bytes.truncate(size as usize);
        }
        Ok(())
    }
    async fn remove(&self, id: &str) -> Result<()> {
        self.0.borrow_mut().remove(id);
        Ok(())
    }
    fn input(&self, id: &str) -> MediaInput {
        MediaInput::Key(id.to_owned())
    }
    async fn write(&self, id: &str, start: u64, chunks: Body, _length: Option<u64>) -> Result<()> {
        let bytes = chunks.bytes().await?;
        let mut all = self.0.borrow_mut();
        let target = all.entry(id.to_owned()).or_default();
        target.truncate(start as usize);
        target.extend(bytes);
        Ok(())
    }
}

struct Host;
#[async_trait(?Send)]
impl Media for Host {
    fn cover(&self, blob: &Value) -> Result<Value> {
        mm_core::media_files::cover(blob, 800.0, 1000.0)
    }
    async fn prepare(
        &self,
        _input: MediaInput,
        content_type: &str,
        _name: &str,
        _user: &MediaUser,
        id: &str,
    ) -> Result<PreparedMedia> {
        let blob = obj! { "path" => format!("synthetic/{id}/original.png"), "width" => 1600, "height" => 900, "dominantColor" => "#123456", "type" => content_type };
        Ok(PreparedMedia {
            data: obj! { "type" => "File", "blob" => blob }.into(),
            extras: obj! {}.into(),
        })
    }
    async fn send(
        &self,
        _result: &MediaResult,
        _method: &str,
        _range: Option<&str>,
    ) -> Result<Response> {
        unreachable!()
    }
    async fn response(&self, _relative: &str) -> Result<Option<MediaResult>> {
        Ok(None)
    }
    async fn image(&self, _source: &Value) -> Result<Option<AnalysisImage>> {
        Ok(None)
    }
}
#[async_trait(?Send)]
impl Fetch for Host {
    async fn fetch(&self, _request: Request) -> Result<Response> {
        panic!("no network in synthetic tests")
    }
}
#[async_trait(?Send)]
impl Resource for Host {
    async fn fetch(&self, _url: &str, _options: ResourceOptions) -> Result<FetchedResource> {
        panic!("no network in synthetic tests")
    }
}
#[async_trait(?Send)]
impl Screenshot for Host {
    async fn capture(&self, _url: &str, _offline: bool) -> Result<Vec<u8>> {
        panic!("no browser in synthetic tests")
    }
}

fn install() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| runtime::install(&TestRuntime));
    runtime::set_env(HashMap::from([("OFFLINE".into(), "1".into())]));
}

fn auth_db() -> Sqlite {
    let db = Sqlite::open(":memory:", false).unwrap();
    auth_store::install_auth_schema(&db).unwrap();
    db
}

fn who(sub: &str, email: &str) -> Identity {
    Identity {
        sub: sub.into(),
        email: email.into(),
    }
}

fn library_count(db: &Sqlite) -> i64 {
    db.one("SELECT COUNT(*) AS n FROM libraries", &[])
        .unwrap()
        .and_then(|row| row.number("n"))
        .unwrap() as i64
}

fn assert_generated(name: &str) {
    assert!(name.starts_with("u-"), "{name}");
    assert!(name.len() >= 66 && name.len() <= 120, "{name}");
    assert!(
        name.bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'),
        "{name}"
    );
    let hash = &name[name.len() - 64..];
    assert!(
        hash.bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
        "{name}"
    );
    assert_eq!(&name[name.len() - 65..name.len() - 64], "-");
}

fn open_library(profile: &MemberProfile) -> (Rc<Store>, Api) {
    install();
    let db: Rc<dyn Sql> = Rc::new(Sqlite::open(":memory:", false).unwrap());
    let store = Rc::new(Store::new(Rc::clone(&db)).unwrap());
    assert!(store_member::ensure_member(&store, profile).unwrap());
    assert!(!store_member::ensure_member(&store, profile).unwrap());
    let api = block_on(Api::new(ApiOptions {
        store: Rc::clone(&store),
        media: Rc::new(Host),
        upload_storage: Rc::new(Memory::default()),
        screenshot: Rc::new(Host),
        resource: Rc::new(Host),
        fetch: Rc::new(Host),
        readonly: false,
        localize: str::to_owned,
        push: Box::new(|_, _| {}),
        random: runtime::random_bytes,
    }))
    .unwrap();
    (store, api)
}

fn send(api: &Api, method: &str, path: &str, body: &str) -> Response {
    let mut request = Request::new(method, &format!("http://localhost:3000{path}")).unwrap();
    request.body = Body::Bytes(body.as_bytes().to_vec());
    if !body.is_empty() {
        request.headers.set("content-type", "application/json");
    }
    block_on(api.handle(&mut request))
        .unwrap()
        .expect("route handled")
}

fn body_text(response: Response) -> String {
    String::from_utf8(block_on(response.body.bytes()).unwrap()).unwrap()
}

#[test]
fn alchemy_probe_repairs_its_mapping_and_initializes_uploads() {
    let probe = include_str!("../../../../infra/alchemy/probe.mjs");
    let constant = |name| {
        probe
            .lines()
            .find_map(|line| line.strip_prefix(&format!("const {name} = '")))
            .and_then(|value| value.strip_suffix("';"))
            .expect("probe fixture constant")
    };
    let username = constant("USER");
    let library = constant("LIBRARY");
    let profile = store_member::profile_from_session(&format!("password:{username}"), username);
    assert_eq!(library, auth_store::library_name(&profile.account));
    assert!(auth_store::generated_library(&profile.account, library));

    let db = auth_db();
    let identity = who(&profile.account, username);
    auth_store::set_library(&db, &profile.account, "alchemy-staging-synthetic").unwrap();
    let previous = auth_store::library_for_session(&db, &identity).unwrap();
    assert!(!auth_store::generated_library(&profile.account, &previous));
    auth_store::set_library(&db, &profile.account, library).unwrap();
    assert_eq!(auth_store::library_for_session(&db, &identity).unwrap(), library);

    let (_store, api) = open_library(&profile);
    let response = send(&api, "POST", "/uploads/authorize", "");
    assert_eq!(response.status, 200);
    let capability = json::parse(&body_text(response)).unwrap();
    assert!(capability.get("url").as_str().unwrap().starts_with("/uploads/"));
    assert!(capability.get("token").as_str().is_some_and(|value| !value.is_empty()));
}

#[test]
fn first_sign_in_creates_a_library_and_the_second_reuses_it() {
    let db = auth_db();
    auth_store::set_library(&db, "google:owner@example.com", "owner").unwrap();
    let owner = who("google-subject", "Owner@Example.com");
    assert_eq!(
        auth_store::library_for_session(&db, &owner).unwrap(),
        "owner"
    );
    assert_eq!(
        auth_store::library_for_session(&db, &owner).unwrap(),
        "owner"
    );
    assert_eq!(library_count(&db), 1);

    let google = who("1099", "Ada@OctavianTocan.com");
    let first = auth_store::library_for_session(&db, &google).unwrap();
    let second = auth_store::library_for_session(&db, &google).unwrap();
    assert_eq!(first, second);
    assert_eq!(
        first,
        auth_store::library_name(&auth_store::account_of(&google))
    );
    assert_generated(&first);
    assert!(first.contains("ada-octaviantocan-com"));
    assert!(auth_store::generated_library(
        &auth_store::account_of(&google),
        &first
    ));
    assert!(!auth_store::generated_library(
        "google:owner@example.com",
        "owner"
    ));

    let password = who("password:ada", "ada");
    let password_library = auth_store::library_for_session(&db, &password).unwrap();
    assert_eq!(
        password_library,
        auth_store::library_for_session(&db, &password).unwrap()
    );
    assert_ne!(password_library, first);
    assert!(password_library.contains("-ada-") || password_library.contains("u-ada-"));
    assert_generated(&password_library);
    assert_eq!(library_count(&db), 3);

    let dotted = auth_store::library_name("google:ada.lovelace@x.com");
    let dashed = auth_store::library_name("google:ada-lovelace@x.com");
    assert_ne!(dotted, dashed, "same slug, different account");
    assert_generated(&dotted);
    assert_generated(&dashed);

    let taken = auth_store::library_name("google:new@octaviantocan.com");
    auth_store::set_library(&db, "password:reserved", &taken).unwrap();
    let moved = auth_store::claim_library(&db, "google:new@octaviantocan.com").unwrap();
    assert_eq!(moved, format!("{taken}-2"));
    assert!(auth_store::generated_library(
        "google:new@octaviantocan.com",
        &moved
    ));
    assert_eq!(
        auth_store::library_of(&db, &who("password:reserved", "reserved"))
            .unwrap()
            .as_deref(),
        Some(taken.as_str())
    );
    assert_eq!(
        auth_store::claim_library(&db, "google:new@octaviantocan.com").unwrap(),
        moved
    );
    assert!(auth_store::claim_library(&db, "").is_err());
}

#[test]
fn new_library_saves_a_card_and_cannot_read_the_owner_card() {
    let owner_profile = MemberProfile {
        account: "google:owner@example.com".into(),
        email: "owner@example.com".into(),
        name: "owner".into(),
    };
    let (owner_store, owner_api) = open_library(&owner_profile);
    assert!(
        !store_member::ensure_member(
            &owner_store,
            &MemberProfile {
                account: "password:intruder".into(),
                email: "intruder".into(),
                name: "intruder".into()
            }
        )
        .unwrap()
    );
    assert_eq!(
        owner_store
            .user(None)
            .unwrap()
            .unwrap()
            .settings
            .get("email")
            .as_str(),
        Some("owner@example.com")
    );

    let owner_page = send(&owner_api, "GET", "/everything", "");
    assert_eq!(owner_page.status, 200);
    let owner_html = body_text(owner_page);
    assert!(owner_html.contains("<mymind-grid"));
    let owner_card = send(
        &owner_api,
        "POST",
        "/objects",
        &json::to_string(&obj! { "content" => "Owner note" }.into()),
    );
    assert_eq!(owner_card.status, 201);
    let owner_id = json::parse(&body_text(owner_card))
        .unwrap()
        .get("id")
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(owner_id.len(), 22);

    let fresh = store_member::profile_from_session("password:newbie", "newbie");
    assert_eq!(fresh.account, "password:newbie");
    assert_eq!(fresh.name, "newbie");
    assert_eq!(fresh.email, "newbie");
    let google = store_member::profile_from_session("1099", "Ada@OctavianTocan.com");
    assert_eq!(google.account, "google:ada@octaviantocan.com");
    assert_eq!(google.name, "ada");
    assert_eq!(google.email, "ada@octaviantocan.com");

    let (store, api) = open_library(&fresh);
    let user = store.user(None).unwrap().unwrap();
    assert_eq!(user.id_prefix.len(), 11);
    assert_eq!(user.settings.get("name").as_str(), Some("newbie"));
    assert_eq!(user.settings.get("objectLimit").as_f64(), Some(50_000.0));
    assert!(store.grid_cards(None, 0, None).unwrap().is_empty());

    let page = send(&api, "GET", "/everything", "");
    assert_eq!(page.status, 200);
    let html = body_text(page);
    assert!(html.contains("<mymind-grid"));
    assert!(html.contains("prefers-color-scheme: dark"));
    assert!(html.contains(&user.id));
    assert!(!html.contains(&owner_id));

    let created = send(
        &api,
        "POST",
        "/objects",
        &json::to_string(&obj! { "content" => "First note" }.into()),
    );
    assert_eq!(created.status, 201);
    let id = json::parse(&body_text(created))
        .unwrap()
        .get("id")
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(store.card(&id, None).unwrap().unwrap().kind, "Note");
    assert_eq!(
        send(&api, "GET", &format!("/app/v1/cards/{id}"), "").status,
        200
    );

    let missing = send(&api, "GET", &format!("/app/v1/cards/{owner_id}"), "");
    assert_eq!(missing.status, 404);
    assert!(store.card(&owner_id, None).unwrap().is_none());
    assert!(owner_store.card(&id, None).unwrap().is_none());
    assert_eq!(
        send(&owner_api, "GET", &format!("/app/v1/cards/{id}"), "").status,
        404
    );
}
