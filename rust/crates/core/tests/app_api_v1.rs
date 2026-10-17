//! The app's top-of-mind route and the `x-upload-token` header of the upload PUT.

mod common;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

use async_trait::async_trait;
use common::{temp_copy, test_db};
use futures_executor::block_on;
use mm_core::app_api::AppApi;
use mm_core::host::{AnalysisImage, Media, MediaInput, MediaResult, MediaUser, PreparedMedia, UploadStorage};
use mm_core::http::{Body, Headers, Response};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::search::Search;
use mm_core::sql::SqlValue;
use mm_core::store::Store;
use mm_core::uploads::{UploadOptions, UploadRequest, Uploads};
use mm_core::{Result, Value};
use mm_js::{json, url};
use mm_sqlite::Sqlite;

struct TestRuntime;

impl Runtime for TestRuntime {
    fn spawn(&self, task: Task) {
        block_on(task);
    }

    fn sleep(&self, _ms: u64) -> Task {
        Box::pin(std::future::pending())
    }

    fn log(&self, _line: &str) {}
}

fn install() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| runtime::install(&TestRuntime));
    runtime::set_env(HashMap::new());
}

fn open() -> Rc<Store> {
    install();
    let copy = temp_copy(&test_db());
    Rc::new(Store::new(Rc::new(Sqlite::open(&copy, false).unwrap())).unwrap())
}

struct NoMedia;

#[async_trait(?Send)]
impl Media for NoMedia {
    fn cover(&self, _blob: &Value) -> Result<Value> {
        unreachable!()
    }
    async fn prepare(&self, _input: MediaInput, _content_type: &str, _name: &str, _user: &MediaUser, _id: &str) -> Result<PreparedMedia> {
        unreachable!()
    }
    async fn send(&self, _result: &MediaResult, _method: &str, _range: Option<&str>) -> Result<Response> {
        unreachable!()
    }
    async fn response(&self, _relative: &str) -> Result<Option<MediaResult>> {
        Ok(None)
    }
    async fn image(&self, _blob_or_cover: &Value) -> Result<Option<AnalysisImage>> {
        Ok(None)
    }
}

#[derive(Default)]
struct Memory(RefCell<HashMap<String, Vec<u8>>>);

#[async_trait(?Send)]
impl UploadStorage for Memory {
    fn key(&self) -> String {
        "memory-uploads".into()
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

fn chunk(extra: &[(&str, &str)]) -> UploadRequest {
    let mut pairs = vec![("content-range", "bytes 0-4/5"), ("content-type", "text/plain"), ("x-file-name", "a.txt")];
    pairs.extend_from_slice(extra);
    UploadRequest { headers: Headers::from_pairs(pairs), body: Body::Bytes(b"hello".to_vec()) }
}

fn status(result: Result<Value>) -> u16 {
    match result {
        Ok(_) => 200,
        Err(error) => error.http_status().expect("status error"),
    }
}

async fn authorized(uploads: &Uploads) -> (String, String) {
    let found = uploads.authorize(&Value::Null).await.unwrap();
    let path = found.get("url").as_str().unwrap().rsplit('/').next().unwrap().to_owned();
    (path, found.get("token").as_str().unwrap().to_owned())
}

#[test]
fn upload_token_header_is_checked_like_the_bearer() {
    let store = open();
    let uploads = Uploads::new(store, Rc::new(Memory::default()), Rc::new(NoMedia), UploadOptions::default()).unwrap();
    block_on(async {
        let (id, token) = authorized(&uploads).await;
        let app = "Bearer app-token-not-an-upload-token";
        let wrong = "0".repeat(token.len());
        assert_eq!(status(uploads.put(&id, chunk(&[("authorization", app)])).await), 401, "app token alone");
        assert_eq!(status(uploads.put(&id, chunk(&[])).await), 401, "no token");
        assert_eq!(status(uploads.put(&id, chunk(&[("authorization", app), ("x-upload-token", &wrong)])).await), 401, "wrong header token");
        let bearer = format!("Bearer {token}");
        assert_eq!(status(uploads.put(&id, chunk(&[("authorization", &bearer), ("x-upload-token", &wrong)])).await), 401, "header wins over bearer");
        assert_eq!(status(uploads.put(&id, chunk(&[("x-upload-token", &token), ("authorization", app)])).await), 200, "app token plus header");
        let (second, token) = authorized(&uploads).await;
        let bearer = format!("Bearer {token}");
        assert_eq!(status(uploads.put(&second, chunk(&[("authorization", &bearer)])).await), 200, "web path unchanged");
    });
}

#[test]
fn top_of_mind_lists_pins_in_order() {
    let store = open();
    let db = store.db();
    let ids: Vec<String> = db
        .query("SELECT id FROM cards WHERE deleted_ns IS NULL AND (flags & 393224) = 0 AND (flags & 512) = 0 ORDER BY id LIMIT 3", &[])
        .unwrap()
        .iter()
        .map(|row| row.text("id").unwrap_or("").to_owned())
        .collect();
    assert_eq!(ids.len(), 3);
    db.execute("DELETE FROM pins", &[]).unwrap();
    for (position, id) in [(0, &ids[2]), (1, &ids[0]), (2, &ids[1])] {
        db.execute("UPDATE cards SET flags = flags | 512 WHERE id = ?", &[SqlValue::from(id.as_str())]).unwrap();
        db.execute("INSERT INTO pins VALUES (?, ?)", &[SqlValue::from(id.as_str()), SqlValue::from(position as f64)]).unwrap();
    }
    install();
    let search = Rc::new(Search::new(store.clone(), None, false).unwrap());
    let api = AppApi::new(store, search, |text| text.to_owned());
    let get = |target: &str| -> Result<Value> {
        let address = url::parse(target, Some("http://localhost")).unwrap();
        let response = block_on(api.handle("GET", &address))?.expect("answered");
        assert_eq!(response.status, 200);
        Ok(json::parse(&String::from_utf8(block_on(response.body.bytes())?).unwrap())?)
    };
    let page = get("/app/v1/top-of-mind").unwrap();
    let listed: Vec<String> = page.get("cards").as_array().unwrap().iter().map(|card| card.get("id").as_str().unwrap().to_owned()).collect();
    assert_eq!(listed[..3], vec![ids[2].clone(), ids[0].clone(), ids[1].clone()][..]);
    let first = get("/app/v1/top-of-mind?limit=2").unwrap();
    assert_eq!(first.get("cards").as_array().unwrap().len(), 2);
    let next = first.get("next").as_str().unwrap().to_owned();
    let rest = get(&format!("/app/v1/top-of-mind?limit=2&cursor={next}")).unwrap();
    assert_eq!(rest.get("cards").as_array().unwrap()[0].get("id").as_str(), Some(ids[1].as_str()));
    let all = get("/app/v1/top-of-mind?limit=200").unwrap();
    assert!(all.get("next").is_nullish());
    let post = block_on(api.handle("POST", &url::parse("/app/v1/top-of-mind", Some("http://localhost")).unwrap())).unwrap().unwrap();
    assert_eq!(post.status, 405);
}
