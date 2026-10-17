//! `mymind search-recordings` against `compareRecordings` of `tools/search-recordings.mjs` (which has no command
//! line of its own, so `tests/drivers/tool-search-recordings.mjs` prints its rows), over the owner fixture.

#[path = "../../core/tests/common/mod.rs"]
mod common;
#[path = "tools_common/mod.rs"]
mod tools_common;

use tools_common::{run_node, run_rust, tmproot};

const DRIVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/drivers/tool-search-recordings.mjs");

#[test]
fn compares_recordings_as_compare_recordings() {
    let (js_root, rust_root) = (tmproot("search-recordings-js"), tmproot("search-recordings"));
    let copy = common::temp_copy(&common::test_db());
    let js = run_node(&js_root, DRIVER, &[copy.to_str().unwrap()], &[]);
    let rust = run_rust(&rust_root, "search-recordings", &["--db", common::test_db().to_str().unwrap()], &[], None);
    assert_eq!(js.status, 0, "{}", js.stderr);
    assert!(js.stdout.contains("\"q\": \"design\""));
    assert_eq!((rust.status, rust.stdout, rust.stderr), (0, js.stdout, String::new()));
}

#[test]
fn a_missing_recording_fails_with_status_one() {
    let (js_root, rust_root) = (tmproot("search-recordings-no-js"), tmproot("search-recordings-no"));
    for root in [&js_root, &rust_root] {
        std::fs::remove_file(root.join("replay")).unwrap();
        std::fs::create_dir_all(root.join("replay/bodies")).unwrap();
        std::fs::write(root.join("replay/index.json"), "{}").unwrap();
    }
    let copy = common::temp_copy(&common::test_db());
    let js = run_node(&js_root, DRIVER, &[copy.to_str().unwrap()], &[]);
    let rust = run_rust(&rust_root, "search-recordings", &["--db", common::test_db().to_str().unwrap()], &[], None);
    assert_eq!((js.status, rust.status, rust.stdout.as_str()), (1, 1, ""));
    assert!(rust.stderr.contains("Missing recorded de prefix search"), "{}", rust.stderr);
}
