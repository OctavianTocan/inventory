//! Compare Rust with frozen pre-switch expectations and immutable synthetic fixtures.
//! command, on a fresh root per side.

#[path = "tools_common/mod.rs"]
mod tools_common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use mm_core::auth_store::check_password;
use mm_core::sql::Sql;
use mm_sqlite::Sqlite;
use tools_common::{Out, both};

fn show(out: &Out, exact_stderr: bool) -> String {
    let stdout = out.stdout.lines().map(|line| line.split_once("\tadded ").map_or(line.to_owned(), |(name, _)| format!("{name}\tadded <time>"))).collect::<Vec<_>>().join("\n");
    let stderr = if exact_stderr { out.stderr.clone() } else { String::from("<not compared>") };
    format!("status {}\nstdout {stdout:?}\nstderr {stderr:?}", out.status)
}

fn users(root: &Path) -> Vec<String> {
    let path = root.join("auth.db");
    if !path.exists() {
        return vec!["no database".into()];
    }
    let db = Sqlite::open(&path, true).unwrap();
    db.query("SELECT username FROM users ORDER BY username", &[]).unwrap().iter().map(|row| row.text("username").unwrap().to_owned()).collect()
}

fn env_of(root: &Path) -> String {
    let path = root.join("fixture-vars");
    let Ok(text) = std::fs::read_to_string(&path) else { return "no .env".into() };
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    let masked = text.lines().map(|line| if line.starts_with("TEST_USER_PASSWORD=") || line.starts_with("OTHER_PASSWORD=") { format!("{}=<random>", line.split('=').next().unwrap()) } else { line.to_owned() }).collect::<Vec<_>>().join("\n");
    format!("mode {mode:o}\n{masked}")
}

fn env_password(root: &Path, key: &str) -> String {
    let text = std::fs::read_to_string(root.join("fixture-vars")).unwrap();
    text.lines().find_map(|line| line.strip_prefix(&format!("{key}="))).unwrap().to_owned()
}

fn accepts(root: &Path, user: &str, password: &str) -> bool {
    let db = Sqlite::open(root.join("auth.db"), true).unwrap();
    check_password(&db, user, password).unwrap().is_some()
}

#[test]
fn manages_users_as_auth_user() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_auth_user.rs")));
    let (js, rust) = both("auth-user", |root, run| {
        let db = root.join("auth.db");
        let db = db.to_str().unwrap();
        let mut log = Vec::new();
        macro_rules! step {
            ($name:expr, $out:expr, $exact:expr) => {
                log.push(format!("{}\n{}", $name, show(&$out, $exact)))
            };
        }
        step!("no arguments", run(&["--db", db], &[], None), true);
        step!("add without a name", run(&["add", "--db", db], &[], None), true);
        step!("remove without a name", run(&["remove", "--db", db], &[], None), true);
        step!("unknown command", run(&["frobnicate", "--db", db], &[], None), true);
        step!("unknown option", run(&["list", "--bogus"], &[], None), false);
        step!("missing option value", run(&["list", "--db"], &[], None), false);
        step!("list before any user", run(&["list", "--db", db], &[], None), true);
        step!("add alice", run(&["add", "alice", "--db", db], &[], Some("secret-password-1234\n")), true);
        assert!(accepts(root, "alice", "secret-password-1234"));
        step!("add alice again", run(&["add", "alice", "--db", db], &[], Some("other-password-12345\r\n")), true);
        assert!(accepts(root, "alice", "other-password-12345") && !accepts(root, "alice", "secret-password-1234"));
        step!("add Bob (name is normalised)", run(&["add", "Bob", "--db", db], &[], Some("no newline at the end")), true);
        assert!(accepts(root, "bob", "no newline at the end"));
        step!("add with a short password", run(&["add", "erin", "--db", db], &[], Some("short\n")), true);
        step!("add with an empty password", run(&["add", "erin", "--db", db], &[], Some("")), true);
        step!("list", run(&["list", "--db", db], &[], None), true);
        step!("remove a missing user", run(&["remove", "nobody", "--db", db], &[], None), true);
        step!("remove bob", run(&["remove", "bob", "--db", db], &[], None), true);
        step!("add through AUTH_DB", run(&["add", "dora"], &[("AUTH_DB", db)], Some("dora-password-12345\n")), true);
        step!("list through AUTH_DB", run(&["list"], &[("AUTH_DB", db)], None), true);
        step!("bad prefix", run(&["add", "carol", "--save-env", "lower", "--db", db], &[], None), true);
        step!("save-env, no .env yet", run(&["add", "carol", "--save-env", "TEST_USER", "--db", db], &[], None), true);
        assert!(accepts(root, "carol", &env_password(root, "TEST_USER_PASSWORD")));
        log.push(format!("env after the first save\n{}", env_of(root)));
        std::fs::write(root.join("fixture-vars"), "KEEP=1\nTEST_USER_NAME=old\n\nOTHER=2\nTEST_USER_PASSWORD=stale\n").unwrap();
        step!("save-env over an existing .env", run(&["add", "Frank", "--save-env", "TEST_USER", "--db", db], &[], None), true);
        assert!(accepts(root, "frank", &env_password(root, "TEST_USER_PASSWORD")));
        log.push(format!("env after the second save\n{}", env_of(root)));
        step!("save-env with another prefix", run(&["add", "gina", "--save-env", "OTHER", "--db", db], &[], None), true);
        assert!(accepts(root, "gina", &env_password(root, "OTHER_PASSWORD")));
        log.push(format!("env after the third save\n{}", env_of(root)));
        log.push(format!("users {:?}", users(root)));
        log
    });
    assert_eq!(rust, js);
}
