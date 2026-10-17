//! `mymind auth-user`: the port of `tools/auth-user.mjs`, which manages the password users who can sign in on a
//! server with sign-in.

use std::io::{IsTerminal, Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use mm_core::auth_store::{install_auth_schema, list_users, remove_user, set_password};
use mm_core::sql::Sql;
use mm_core::{Error, Result, runtime};
use mm_js::{base64, date};
use mm_sqlite::Sqlite;

use crate::parse_args::{Kind, Opt, parse};

const USAGE: &str = "usage: node tools/auth-user.mjs add <username> [--save-env PREFIX] | remove <username> | list";

// Set `entries` in the root's .env, replacing earlier values and keeping the file private (`saveEnv`).
fn save_env(root: &Path, entries: &[(String, String)]) -> std::io::Result<()> {
    let file = crate::env_file::path(root);
    let all: Vec<String> = match std::fs::read(&file) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).split('\n').map(str::to_owned).collect(),
        Err(_) => Vec::new(),
    };
    let last = all.len().saturating_sub(1);
    let mut lines: Vec<String> = all.into_iter().enumerate().filter(|(index, line)| !line.is_empty() || *index < last).map(|(_, line)| line).collect();
    lines.retain(|line| !entries.iter().any(|(key, _)| line.starts_with(&format!("{key}="))));
    lines.extend(entries.iter().map(|(key, value)| format!("{key}={value}")));
    let mut out = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&file)?;
    out.write_all(format!("{}\n", lines.join("\n")).as_bytes())?;
    drop(out);
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600))
}

// The password piped to stdin, without its trailing newline (`stdinPassword`).
fn stdin_password() -> Result<String> {
    if std::io::stdin().is_terminal() {
        return Err(Error::internal("pipe the password to stdin, or use --save-env to generate one"));
    }
    let mut bytes = Vec::new();
    std::io::stdin().read_to_end(&mut bytes).map_err(|error| Error::internal(error.to_string()))?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(text.strip_suffix("\r\n").or_else(|| text.strip_suffix('\n')).unwrap_or(&text).to_owned())
}

fn valid_prefix(prefix: &str) -> bool {
    let mut bytes = prefix.bytes();
    bytes.next().is_some_and(|first| first.is_ascii_uppercase()) && bytes.all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn command(db: &dyn Sql, root: &Path, command: Option<&str>, username: Option<&str>, prefix: Option<&str>) -> Result<u8> {
    match (command, username.filter(|name| !name.is_empty())) {
        (Some("add"), Some(username)) => {
            let prefix = prefix.filter(|prefix| !prefix.is_empty());
            if prefix.is_some_and(|prefix| !valid_prefix(prefix)) {
                return Err(Error::internal("--save-env takes an uppercase prefix such as TEST_USER"));
            }
            let password = match prefix {
                Some(_) => base64::encode_url(&runtime::random_bytes(24)),
                None => stdin_password()?,
            };
            let name = set_password(db, username, &password)?;
            if let Some(prefix) = prefix {
                save_env(root, &[(format!("{prefix}_NAME"), name.clone()), (format!("{prefix}_PASSWORD"), password)]).map_err(|error| Error::internal(error.to_string()))?;
            }
            let note = prefix.map_or_else(String::new, |prefix| format!("; {prefix}_NAME and {prefix}_PASSWORD are in .env"));
            println!("{name} can sign in at /signin/password{note}.");
            Ok(0)
        }
        (Some("remove"), Some(username)) => {
            if remove_user(db, username)? {
                println!("{username} removed, and signed out everywhere.");
            } else {
                println!("No password user {username}.");
            }
            Ok(0)
        }
        (Some("list"), _) => {
            for user in list_users(db)? {
                let when = date::iso_from_ms(user.created_ms).ok_or_else(|| Error::internal("Invalid time value"))?;
                println!("{}\tadded {when}", user.username);
            }
            Ok(0)
        }
        _ => {
            eprintln!("{USAGE}");
            Ok(2)
        }
    }
}

/// Run `tools/auth-user.mjs` (`add`, `remove` or `list` of password users).
///
/// # Arguments
/// * `root` - The repository root, where `.env` and the default `data/auth.db` live.
/// * `args` - The arguments after the subcommand name: `add <username> [--save-env PREFIX] | remove <username> | list`,
///   with `--db` (default `AUTH_DB` or `data/auth.db`).
///
/// # Returns
/// The exit status: 0, 2 for a usage error, 1 for a failure.
pub async fn run(root: &Path, args: &[String]) -> u8 {
    mm_core::runtime::set_env(std::env::vars().collect());
    let declared = [Opt { name: "save-env", kind: Kind::Text }, Opt { name: "db", kind: Kind::Text }];
    let parsed = match parse(args, &declared, true) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
    };
    let db_path = parsed
        .text("db")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("AUTH_DB").map(PathBuf::from))
        .unwrap_or_else(|| root.join("data").join("auth.db"));
    let db = match Sqlite::open(&db_path, false).map(Rc::new).and_then(|db| install_auth_schema(&*db).map(|()| db)) {
        Ok(db) => db,
        Err(error) => {
            eprintln!("Error: {error}");
            return 1;
        }
    };
    match command(&*db, root, parsed.positionals.first().map(String::as_str), parsed.positionals.get(1).map(String::as_str), parsed.text("save-env")) {
        Ok(status) => status,
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}
