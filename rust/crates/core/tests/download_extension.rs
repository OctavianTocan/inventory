//! The Chrome extension on `/download`: its section of the page and its media type.

use mm_core::download_page::{EXTENSION_TYPE, content_type, download_page};
use mm_js::{Value, json};

fn linux() -> Value {
    json::parse(r#"{"name":"mymind-linux-x86_64.tar.gz","label":"Linux (x86_64)","bytes":52428800,"built":"2026-10-17T01:02:03Z","sha256":"ab"}"#).unwrap()
}

fn apk() -> Value {
    json::parse(r#"{"name":"mymind-0.2.0.apk","label":"Android","platform":"android","bytes":10485760,"built":"2026-10-18T09:00:00Z","sha256":"cd","versionName":"0.2.0","versionCode":2,"notes":[]}"#).unwrap()
}

fn extension(version: &str, built: &str) -> Value {
    json::parse(&format!(
        r#"{{"name":"mymind-extension-{version}.zip","label":"Chrome <extension>","platform":"chrome-extension","bytes":1048576,"built":"{built}","sha256":"ef","version":"{version}"}}"#
    ))
    .unwrap()
}

fn manifest(files: Vec<Value>) -> Value {
    mm_js::obj! { "library" => "https://library.example", "files" => Value::Array(files) }.into()
}

#[test]
fn the_page_lists_the_extension_between_android_and_linux() {
    let page = download_page(&manifest(vec![linux(), extension("1.0.0", "2026-10-17T20:00:00Z"), apk()]), "owner@example.com").unwrap();
    let android = page.find("<h2>Android</h2>").expect("an Android section");
    let chrome = page.find("<h2>Chrome extension</h2>").expect("an extension section");
    let linux = page.find("<h2>Linux</h2>").expect("a Linux heading");
    assert!(android < chrome && chrome < linux && linux < page.find("mymind-linux-x86_64.tar.gz").unwrap(), "{page}");
    assert!(page.contains("<a href=\"/download/mymind-extension-1.0.0.zip\">Chrome &lt;extension&gt;</a>"), "{page}");
    assert!(page.contains("<span>Version 1.0.0 · 1.0 MB · built 2026-10-17</span><code>sha256 ef</code>"), "{page}");
    assert!(page.contains("Load unpacked"), "{page}");
    assert_eq!(page.matches("mymind-extension-1.0.0.zip").count(), 1, "the extension is not listed twice");
}

#[test]
fn the_newest_extension_is_listed() {
    let page = download_page(&manifest(vec![extension("1.1.0", "2026-10-18T00:00:00Z"), extension("1.0.0", "2026-10-17T00:00:00Z")]), "owner@example.com").unwrap();
    assert!(page.contains("mymind-extension-1.1.0.zip") && !page.contains("mymind-extension-1.0.0.zip"), "{page}");
}

#[test]
fn an_extension_only_manifest_says_the_linux_app_is_missing() {
    let page = download_page(&manifest(vec![extension("1.0.0", "2026-10-17T20:00:00Z")]), "owner@example.com").unwrap();
    assert!(!page.contains("<h2>Android</h2>"), "{page}");
    assert!(page.contains("<h2>Chrome extension</h2>"), "{page}");
    assert!(page.contains("<h2>Linux</h2>\n<ul><li>No Linux app has been packaged yet."), "{page}");
}

#[test]
fn the_extension_is_sent_as_a_zip() {
    assert_eq!(content_type(&extension("1.0.0", "2026-10-17T20:00:00Z")), EXTENSION_TYPE);
    assert_eq!(EXTENSION_TYPE, "application/zip");
}
