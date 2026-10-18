//! `lib/auth-store.mjs` against the Rust port: scrypt hashes for fixed salts, then the same operation sequence on a
//! database each, plus password checks across the two databases.

mod common;

use mm_core::auth_store::{self, Identity, hash_password};
use mm_core::sql::{Sql, SqlValue};
use mm_core::{Error, Value, obj};
use mm_js::{base64, date};
use mm_sqlite::Sqlite;

// Logical identity for a frozen pre-switch comparison.
const DRIVER: &str = "driver:core:auth_store";
const NOW: i64 = 1_760_000_000_000;

fn s(text: &str) -> Value {
    Value::String(text.to_owned())
}

fn hex(text: &str) -> Vec<u8> {
    (0..text.len() / 2).map(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16).unwrap()).collect()
}

fn scratch(name: &str) -> std::path::PathBuf {
    let directory = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("auth-store-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    directory.join("auth.db")
}

#[test]
fn scrypt_hashes_match() {
    let _goldens = common::test_guard();
    let passwords = ["", "password", "correct horse battery staple", "pässwörd ✓ 日本語", "a\u{0}b", "\u{1F600}x", &"long".repeat(500)];
    let salts = ["", "00", "00112233445566778899aabbccddeeff", "ffffffffffffffffffffffffffffffff", &"ab".repeat(64)];
    let mut pairs = Vec::new();
    for password in passwords {
        for salt in salts {
            pairs.push(Value::Array(vec![s(password), s(salt)]));
        }
    }
    let js = common::call(DRIVER, "hashes", vec![Value::Array(pairs.clone())]);
    let actual = Value::Array(pairs.iter().map(|pair| s(&base64::hex(&hash_password(&pair.at(0).to_js_string(), &hex(&pair.at(1).to_js_string()))))).collect());
    common::assert_value(&actual, &js, "fixed-salt scrypt hashes");
    println!("auth_store: {} hashes compared", pairs.len());
}

fn class(error: &Error) -> (String, String) {
    match error {
        Error::Invalid(message) => ("invalid".into(), message.clone()),
        Error::Internal(message) => ("internal".into(), message.clone()),
        Error::Status { message, .. } => ("status".into(), message.clone()),
    }
}

fn identity(sub: &str, email: &str) -> Identity {
    Identity { sub: sub.into(), email: email.into() }
}

fn rows(db: &Sqlite, sql: &str) -> Value {
    Value::Array(
        db.query(sql, &[])
            .unwrap()
            .iter()
            .map(|row| {
                let mut object = mm_core::Object::new();
                for (name, value) in row.to_object().iter() {
                    object.insert(name.into_owned(), if let Value::Bytes(bytes) = value { s(&base64::hex(bytes)) } else { value.clone() });
                }
                Value::Object(object)
            })
            .collect(),
    )
}

fn id_value(who: &Identity) -> Value {
    obj! { "sub" => who.sub.as_str(), "email" => who.email.as_str() }.into()
}

#[test]
fn operations_match() {
    let _goldens = common::test_guard();
    let ops: Vec<Vec<Value>> = vec![
        vec![s("subject"), s("  Alice ")],
        vec![s("set"), s("Alice"), s("correct horse battery")],
        vec![s("set"), s("  bob.smith@x-y_z "), s("another long password!")],
        vec![s("set"), s("short"), s("tooshort")],
        vec![s("set"), s(""), s("long enough password 1")],
        vec![s("set"), s("a b"), s("long enough password 1")],
        vec![s("set"), s("UPPER"), s("long enough password 1")],
        vec![s("set"), s(&"x".repeat(65)), s("long enough password 1")],
        vec![s("set"), s(&"x".repeat(64)), s("long enough password 1")],
        vec![s("set"), s("ünï"), s("long enough password 1")],
        vec![s("set"), s("emoji"), s("😀😀😀😀😀😀😀😀")],
        vec![s("set"), s("emoji"), s("😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀")],
        vec![s("set"), s("alice"), s("a second password!!")],
        vec![s("check"), s("alice"), s("a second password!!")],
        vec![s("check"), s("ALICE "), s("a second password!!")],
        vec![s("check"), s("alice"), s("correct horse battery")],
        vec![s("check"), s("nobody"), s("correct horse battery")],
        vec![s("check"), s(""), s("")],
        vec![s("check"), s("a b"), s("x")],
        vec![s("check"), s("bob.smith@x-y_z"), s("another long password!")],
        vec![s("check"), s("emoji"), s("😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀")],
        vec![s("list")],
        vec![s("sql"), s("INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES ('h1', 'password:alice', 'e', 1, 2, 3)")],
        vec![s("sql"), s("INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES ('h2', 'password:bob.smith@x-y_z', 'e', 1, 2, 3)")],
        vec![s("sql"), s("INSERT INTO app_tokens (hash, sub, email, kind, device, created_ms, expires_ms, used_ms) VALUES ('t1', 'password:alice', 'e', 'k', 'd', 1, 2, 3)")],
        vec![s("remove"), s("ALICE")],
        vec![s("remove"), s("alice")],
        vec![s("remove"), s("nobody")],
        vec![s("list")],
        vec![s("account"), id_value(&identity("password:alice", "x@y.z"))],
        vec![s("account"), id_value(&identity("google-1", "Mixed@Case.COM"))],
        vec![s("account"), id_value(&identity("", "İ@x.com"))],
        vec![s("setLibrary"), s("google:owner@example.com"), s("main")],
        vec![s("setLibrary"), s("google:owner@example.com"), s("main-2")],
        vec![s("setLibrary"), s("password:bob"), s("bob-lib")],
        vec![s("setLibrary"), s("x"), s("")],
        vec![s("setLibrary"), s("x"), s("Upper")],
        vec![s("setLibrary"), s("x"), s("with space")],
        vec![s("setLibrary"), s("x"), s("under_score")],
        vec![s("setLibrary"), s("x"), s("é")],
        vec![s("library"), id_value(&identity("g", "OWNER@example.com"))],
        vec![s("library"), id_value(&identity("password:bob", "z"))],
        vec![s("library"), id_value(&identity("password:nobody", "z"))],
        vec![s("library"), id_value(&identity("g", "none@example.com"))],
    ];
    let js_path = scratch("js");
    let rust_path = scratch("rust");
    let _ = std::fs::remove_file(&js_path);
    let _ = std::fs::remove_file(&rust_path);
    let js = common::call(DRIVER, "run", vec![s(js_path.to_str().unwrap()), Value::Array(ops.iter().map(|op| Value::Array(op.clone())).collect()), Value::Number(NOW as f64)]);

    date::set_fixed_now(Some(NOW));
    let db = Sqlite::open(&rust_path, false).unwrap();
    auth_store::install_auth_schema(&db).unwrap();
    let mut answers = Vec::new();
    for op in &ops {
        let name = op[0].to_js_string();
        let text = |index: usize| op[index].to_js_string();
        let who = |index: usize| identity(&op[index].get("sub").to_js_string(), &op[index].get("email").to_js_string());
        let result: Result<Value, Error> = match name.as_str() {
            "set" => auth_store::set_password(&db, &text(1), &text(2)).map(|name| s(&name)),
            "check" => auth_store::check_password(&db, &text(1), &text(2)).map(|name| name.map_or(Value::Null, |name| s(&name))),
            "remove" => auth_store::remove_user(&db, &text(1)).map(Value::Bool),
            "list" => auth_store::list_users(&db).map(|users| Value::Array(users.iter().map(|user| obj! { "username" => user.username.as_str(), "created_ms" => user.created_ms }.into()).collect())),
            "subject" => Ok(s(&auth_store::password_subject(&text(1)))),
            "account" => Ok(s(&auth_store::account_of(&who(1)))),
            "setLibrary" => auth_store::set_library(&db, &text(1), &text(2)).map(|()| Value::Null),
            "library" => auth_store::library_of(&db, &who(1)).map(|library| library.map_or(Value::Null, |library| s(&library))),
            "sql" => db.execute(&text(1), &[] as &[SqlValue]).map(|_| Value::Null),
            other => panic!("unknown op {other}"),
        };
        answers.push(match result {
            Ok(value) => obj! { "ok" => value }.into(),
            Err(error) => {
                let (kind, message) = class(&error);
                obj! { "error" => obj! { "name" => if kind == "invalid" { "TypeError" } else { "Error" }, "message" => message } }.into()
            }
        });
    }
    date::set_fixed_now(None);
    let rust: Value = obj! {
        "answers" => answers,
        "users" => rows(&db, "SELECT username, created_ms FROM users ORDER BY username"),
        "libraries" => rows(&db, "SELECT * FROM libraries ORDER BY account"),
        "sessions" => rows(&db, "SELECT hash, sub FROM sessions ORDER BY hash"),
        "tokens" => rows(&db, "SELECT hash, sub FROM app_tokens ORDER BY hash"),
    }
    .into();
    assert_eq!(rust.get("answers").as_array().unwrap().len(), ops.len(), "every auth operation answered");
    common::assert_value(&rust, &js, "auth operations and every table snapshot");

    let synthetic_path = scratch("synthetic-js");
    let _ = std::fs::remove_file(&synthetic_path);
    reconstruct_js_users(&synthetic_path, &js_path);
    let compatibility_path = if common::recording() { &js_path } else { &synthetic_path };
    let js_db = Sqlite::open(compatibility_path, false).unwrap();
    date::set_fixed_now(Some(NOW));
    for (user, password, expected) in [("bob.smith@x-y_z", "another long password!", true), ("alice", "a second password!!", false), ("emoji", "😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀", true), ("emoji", "wrong", false)] {
        let found = auth_store::check_password(&js_db, user, password).unwrap().is_some();
        assert_eq!(found, expected, "Rust checks the JS database for {user}");
    }
    date::set_fixed_now(None);
    let checks: Vec<Value> = [("bob.smith@x-y_z", "another long password!"), ("emoji", "😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀"), ("emoji", "wrong")].iter().map(|(user, password)| Value::Array(vec![s("check"), s(user), s(password)])).collect();
    let cross = common::call(DRIVER, "run", vec![s(rust_path.to_str().unwrap()), Value::Array(checks), Value::Number(NOW as f64)]);
    let cross_answers: Vec<Value> = [("bob.smith@x-y_z", "another long password!"), ("emoji", "😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀"), ("emoji", "wrong")].iter().map(|(user, password)| {
        obj! { "ok" => auth_store::check_password(&db, user, password).unwrap().map_or(Value::Null, |name| s(&name)) }.into()
    }).collect();
    let cross_value: Value = obj! {
        "answers" => cross_answers.clone(),
        "users" => rust.get("users").clone(),
        "libraries" => rust.get("libraries").clone(),
        "sessions" => rust.get("sessions").clone(),
        "tokens" => rust.get("tokens").clone(),
    }.into();
    common::assert_value(&cross_value, &cross, "JS-compatible checks against Rust database");
    let cross = cross_answers;
    assert_eq!(common::text(&cross[0]), common::text(&obj! { "ok" => "bob.smith@x-y_z" }.into()), "JS checks the Rust database");
    assert_eq!(common::text(&cross[1]), common::text(&obj! { "ok" => "emoji" }.into()));
    assert_eq!(common::text(&cross[2]), common::text(&obj! { "ok" => Value::Null }.into()));
    println!("auth_store: {} operations compared", ops.len());
}

// Rebuild the synthetic JS-compatible password rows using frozen Node scrypt outputs, never the Rust hasher.
fn reconstruct_js_users(path: &std::path::Path, legacy_path: &std::path::Path) {
    let users = [
        ("bob.smith@x-y_z".to_owned(), "another long password!", "00".repeat(16)),
        ("upper".to_owned(), "long enough password 1", "11".repeat(16)),
        ("x".repeat(64), "long enough password 1", "22".repeat(16)),
        ("emoji".to_owned(), "😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀", "33".repeat(16)),
    ];
    let pairs = Value::Array(users.iter().map(|(_, password, salt)| Value::Array(vec![s(password), s(salt)])).collect());
    let (hash_values, expectation) = common::synthetic_call(DRIVER, "hashes", vec![pairs]);
    let hashes = hash_values.as_array().expect("synthetic hashes");
    assert_eq!(hashes.len(), users.len(), "every synthetic user has a Node-derived hash");
    let db = Sqlite::open(path, false).unwrap();
    db.exec_batch("CREATE TABLE IF NOT EXISTS users (username TEXT PRIMARY KEY, salt BLOB NOT NULL, hash BLOB NOT NULL,
    created_ms INTEGER NOT NULL) STRICT").unwrap();
    if common::recording() {
        let legacy = Sqlite::open(legacy_path, true).unwrap();
        assert_eq!(common::text(&rows(&db, "PRAGMA table_info(users)")), common::text(&rows(&legacy, "PRAGMA table_info(users)")), "frozen synthetic users schema matches the legacy installer");
    }
    for ((username, password, salt), hash) in users.iter().zip(hashes) {
        db.execute("INSERT INTO users (username, salt, hash, created_ms) VALUES (?, ?, ?, ?)", mm_core::params![username.as_str(), hex(salt), hex(hash.as_str().unwrap()), NOW]).unwrap();
        assert_eq!(auth_store::check_password(&db, username, password).unwrap().as_deref(), Some(username.as_str()), "Rust reads the frozen Node-derived synthetic row");
        assert!(auth_store::check_password(&db, username, "wrong").unwrap().is_none(), "synthetic row refuses another password");
    }
    common::assert_value(&hash_values, &expectation, "Node-derived synthetic scrypt fixture");
}
