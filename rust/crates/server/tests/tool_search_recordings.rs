//! Compare Rust with frozen pre-switch expectations and immutable synthetic fixtures.

#[path = "../../core/tests/common/mod.rs"]
mod common;
#[path = "tools_common/mod.rs"]
mod tools_common;

use tools_common::{run_node, run_rust, tmproot};

// Logical identity for a frozen pre-switch comparison.
const DRIVER: &str = "driver:server:tool-search-recordings";

#[test]
fn compares_recordings_as_compare_recordings() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_search_recordings.rs")));
    let (js_root, rust_root) = (tmproot("search-recordings-js"), tmproot("search-recordings"));
    let copy = common::temp_copy(&common::test_db());
    let js = tools_common::recording().then(|| run_node(&js_root, DRIVER, &[copy.to_str().unwrap()], &[]));
    let rust = run_rust(&rust_root, "search-recordings", &["--db", common::test_db().to_str().unwrap()], &[], None);
    assert_eq!(rust.status, 0, "{}", rust.stderr);
    assert!(rust.stdout.contains("\"q\": \"design\""));
    assert!(rust.stderr.is_empty());
    let actual = format!("status {} stdout {:?} stderr {:?}", rust.status, rust.stdout, rust.stderr);
    let oracle = js.as_ref().map(|out| format!("status {} stdout {:?} stderr {:?}", out.status, out.stdout, out.stderr));
    tools_common::check("recorded prefix comparison", actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
}

#[test]
fn a_missing_recording_fails_with_status_one() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_search_recordings.rs")));
    let (js_root, rust_root) = (tmproot("search-recordings-no-js"), tmproot("search-recordings-no"));
    for root in [&js_root, &rust_root] {
        std::fs::remove_file(root.join("replay")).unwrap();
        std::fs::create_dir_all(root.join("replay/bodies")).unwrap();
        std::fs::write(root.join("replay/index.json"), "{}").unwrap();
    }
    let copy = common::temp_copy(&common::test_db());
    let js = tools_common::recording().then(|| run_node(&js_root, DRIVER, &[copy.to_str().unwrap()], &[]));
    let rust = run_rust(&rust_root, "search-recordings", &["--db", common::test_db().to_str().unwrap()], &[], None);
    assert_eq!((rust.status, rust.stdout.as_str()), (1, ""));
    assert!(rust.stderr.contains("Missing recorded de prefix search"), "{}", rust.stderr);
    let message = "Missing recorded de prefix search";
    let actual = format!("status {} stdout {:?} missing-recording {}", rust.status, rust.stdout, rust.stderr.contains(message));
    let oracle = js.as_ref().map(|out| format!("status {} stdout {:?} missing-recording {}", out.status, out.stdout, out.stderr.contains(message)));
    tools_common::check("missing recorded prefix", actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
}
