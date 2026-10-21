//! Sign-in storage (`lib/auth-store.mjs`): the tables, password users and which library each account reaches.

use std::sync::OnceLock;

use mm_js::text;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::sql::{Sql, SqlExt};
use crate::{Error, Result, params, runtime};

const KEY_BYTES: usize = 64;

static DUMMY: OnceLock<(Vec<u8>, Vec<u8>)> = OnceLock::new();

/// A signed-in session or app token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    /// `sub` stored with the session: a Google subject or `password:<username>`.
    pub sub: String,
    /// The email the session signed in as.
    pub email: String,
}

/// A password user as `listUsers` lists it.
#[derive(Clone, Debug, PartialEq)]
pub struct User {
    /// Stored (lower-case) username.
    pub username: String,
    /// When the user was first added, in milliseconds since the epoch.
    pub created_ms: f64,
}

fn normalize(username: &str) -> String {
    text::to_lower(text::trim(username))
}

fn valid_username(name: &str) -> bool {
    (1..=64).contains(&name.len()) && name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'@' | b'-'))
}

/// Create the sign-in tables when missing (`installAuthSchema`): sessions, app tokens, password users, sign-ins in
/// progress, app nonces, recent password failures and which library each account reaches.
///
/// # Arguments
/// * `db` - Writable database.
///
/// # Errors
/// The database's error.
pub fn install_auth_schema(db: &dyn Sql) -> Result<()> {
    db.exec_batch(
        "CREATE TABLE IF NOT EXISTS sessions (hash TEXT PRIMARY KEY, sub TEXT NOT NULL, email TEXT NOT NULL,
    created_ms INTEGER NOT NULL, expires_ms INTEGER NOT NULL, used_ms INTEGER NOT NULL) STRICT",
    )?;
    db.exec_batch(
        "CREATE TABLE IF NOT EXISTS app_tokens (hash TEXT PRIMARY KEY, sub TEXT NOT NULL, email TEXT NOT NULL, kind TEXT NOT NULL,
    device TEXT NOT NULL, created_ms INTEGER NOT NULL, expires_ms INTEGER NOT NULL, used_ms INTEGER NOT NULL) STRICT",
    )?;
    db.exec_batch(
        "CREATE TABLE IF NOT EXISTS users (username TEXT PRIMARY KEY, salt BLOB NOT NULL, hash BLOB NOT NULL,
    created_ms INTEGER NOT NULL) STRICT",
    )?;
    db.exec_batch(
        "CREATE TABLE IF NOT EXISTS logins (state TEXT PRIMARY KEY, verifier TEXT NOT NULL, nonce TEXT NOT NULL,
    created_ms INTEGER NOT NULL) STRICT",
    )?;
    db.exec_batch("CREATE TABLE IF NOT EXISTS app_nonces (nonce TEXT PRIMARY KEY, created_ms INTEGER NOT NULL) STRICT")?;
    db.exec_batch("CREATE TABLE IF NOT EXISTS failures (key TEXT NOT NULL, attempt TEXT NOT NULL, at_ms INTEGER NOT NULL) STRICT")?;
    db.exec_batch("CREATE INDEX IF NOT EXISTS failures_by_key ON failures (key, at_ms)")?;
    db.exec_batch("CREATE TABLE IF NOT EXISTS libraries (account TEXT PRIMARY KEY, library TEXT NOT NULL) STRICT")
}

/// The session subject that marks a session as signed in with this user's password (`passwordSubject`).
///
/// # Arguments
/// * `username` - Password user, in any case.
///
/// # Returns
/// The `sub` stored with the user's sessions.
pub fn password_subject(username: &str) -> String {
    format!("password:{}", normalize(username))
}

/// The scrypt hash `crypto.scrypt(password, salt, 64)` makes: N 16384, r 8, p 1.
///
/// # Arguments
/// * `password` - Password text, hashed as UTF-8.
/// * `salt` - Salt bytes.
///
/// # Returns
/// The 64-byte hash.
pub fn hash_password(password: &str, salt: &[u8]) -> Vec<u8> {
    let params = scrypt::Params::new(14, 8, 1, KEY_BYTES).expect("the scrypt parameters are valid");
    let mut out = vec![0; KEY_BYTES];
    scrypt::scrypt(password.as_bytes(), salt, &params, &mut out).expect("the output length is valid");
    out
}

/// Add a password user, or give an existing one a new password (`setPassword`).
///
/// # Arguments
/// * `db` - Database holding the `users` table.
/// * `username` - 1-64 characters of letters, digits and `._@-`; stored lower case.
/// * `password` - At least 16 characters.
///
/// # Returns
/// The stored username.
///
/// # Errors
/// An internal error with the JS message when the username or password is not acceptable, or the database's error.
pub fn set_password(db: &dyn Sql, username: &str, password: &str) -> Result<String> {
    let name = normalize(username);
    if !valid_username(&name) {
        return Err(Error::internal("usernames are 1-64 characters of letters, digits and ._@-"));
    }
    if text::len(password) < 16 {
        return Err(Error::internal("passwords need at least 16 characters"));
    }
    let salt = runtime::random_bytes(16);
    let hash = hash_password(password, &salt);
    db.execute(
        "INSERT INTO users (username, salt, hash, created_ms) VALUES (?, ?, ?, ?)
    ON CONFLICT (username) DO UPDATE SET salt = excluded.salt, hash = excluded.hash",
        params![&name, salt, hash, runtime::now_ms()],
    )?;
    Ok(name)
}

/// Check a username and password (`checkPassword`), spending the same time for an unknown user as for a wrong password.
///
/// # Arguments
/// * `db` - Database holding the `users` table.
/// * `username` - Username as typed.
/// * `password` - Password as typed.
///
/// # Returns
/// The stored username when the password is right, otherwise `None`.
///
/// # Errors
/// The database's error, or an invalid-data error when the stored hash is not 64 bytes (`timingSafeEqual` throws).
pub fn check_password(db: &dyn Sql, username: &str, password: &str) -> Result<Option<String>> {
    let name = normalize(username);
    let row = if valid_username(&name) { db.one("SELECT salt, hash FROM users WHERE username = ?", params![&name])? } else { None };
    let stored = row.as_ref().map(|row| (row.blob("salt").unwrap_or_default().to_vec(), row.blob("hash").unwrap_or_default().to_vec()));
    let (salt, hash) = stored.unwrap_or_else(|| DUMMY.get_or_init(|| (runtime::random_bytes(16), runtime::random_bytes(KEY_BYTES))).clone());
    let attempt = hash_password(password, &salt);
    if attempt.len() != hash.len() {
        return Err(Error::invalid("Input buffers must have the same byte length"));
    }
    let equal: bool = attempt.ct_eq(&hash).into();
    Ok((equal && row.is_some()).then_some(name))
}

/// Remove a password user and end their sessions and app tokens (`removeUser`).
///
/// # Arguments
/// * `db` - Database holding the sign-in tables.
/// * `username` - Password user, in any case.
///
/// # Returns
/// Whether the user existed.
///
/// # Errors
/// The database's error.
pub fn remove_user(db: &dyn Sql, username: &str) -> Result<bool> {
    db.execute("DELETE FROM sessions WHERE sub = ?", params![password_subject(username)])?;
    db.execute("DELETE FROM app_tokens WHERE sub = ?", params![password_subject(username)])?;
    Ok(db.execute("DELETE FROM users WHERE username = ?", params![normalize(username)])? > 0)
}

/// The password users by name (`listUsers`).
///
/// # Arguments
/// * `db` - Database holding the `users` table.
///
/// # Errors
/// The database's error.
pub fn list_users(db: &dyn Sql) -> Result<Vec<User>> {
    Ok(db
        .query("SELECT username, created_ms FROM users ORDER BY username", &[])?
        .iter()
        .map(|row| User { username: row.text("username").unwrap_or_default().to_owned(), created_ms: row.number("created_ms").unwrap_or(f64::NAN) })
        .collect())
}

/// The account name an identity has in the libraries table (`accountOf`): `google:<email>` for Google, the session
/// subject for password users.
///
/// # Arguments
/// * `identity` - Signed-in session or app token.
pub fn account_of(identity: &Identity) -> String {
    if identity.sub.starts_with("password:") { identity.sub.clone() } else { format!("google:{}", text::to_lower(&identity.email)) }
}

/// Give an account a library, or move it to another one (`setLibrary`).
///
/// # Arguments
/// * `db` - Database holding the `libraries` table.
/// * `account` - Account name from [`account_of`].
/// * `library` - Library name: lower-case letters, digits and dashes.
///
/// # Errors
/// An internal error with the JS message for a library name that is not acceptable, or the database's error.
pub fn set_library(db: &dyn Sql, account: &str, library: &str) -> Result<()> {
    if library.is_empty() || !library.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-') {
        return Err(Error::internal("library names are lower-case letters, digits and dashes"));
    }
    db.execute("INSERT INTO libraries (account, library) VALUES (?, ?) ON CONFLICT (account) DO UPDATE SET library = excluded.library", params![account, library])?;
    Ok(())
}

/// The library an identity reaches (`libraryOf`).
///
/// # Arguments
/// * `db` - Database holding the `libraries` table.
/// * `identity` - Signed-in session or app token.
///
/// # Returns
/// The library name, or `None` when the account has none.
///
/// # Errors
/// The database's error.
pub fn library_of(db: &dyn Sql, identity: &Identity) -> Result<Option<String>> {
    Ok(db.one("SELECT library FROM libraries WHERE account = ?", params![account_of(identity)])?.and_then(|row| row.text("library").map(str::to_owned)))
}

/// Library name for an account that has none yet: `u-`, a short slug, and the SHA-256 of the account.
///
/// The hash is of the whole account (`google:<email>` or `password:<username>`), so two accounts that slug the same
/// still get different libraries. The name is only lower-case letters, digits and dashes.
///
/// # Arguments
/// * `account` - Account name from [`account_of`].
pub fn library_name(account: &str) -> String {
    let hash = mm_js::base64::hex(&Sha256::digest(account.as_bytes()));
    let slug = library_slug(account);
    if slug.is_empty() { format!("u-{hash}") } else { format!("u-{slug}-{hash}") }
}

/// Whether `library` is the name [`library_name`] would give `account`, including a numeric collision suffix.
///
/// # Arguments
/// * `account` - Account name from [`account_of`].
/// * `library` - Library name stored for that account.
pub fn generated_library(account: &str, library: &str) -> bool {
    let base = library_name(account);
    library == base || library.strip_prefix(&format!("{base}-")).is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit()))
}

/// The library a signed-in account reaches. An account that already has one keeps it; the first request creates one.
///
/// # Arguments
/// * `db` - Database holding the `libraries` table.
/// * `identity` - Signed-in session or app token.
///
/// # Errors
/// The database's error, or an internal error when no free library name can be reserved.
pub fn library_for_session(db: &dyn Sql, identity: &Identity) -> Result<String> {
    if let Some(library) = library_of(db, identity)? {
        return Ok(library);
    }
    claim_library(db, &account_of(identity))
}

/// Reserve [`library_name`] for `account` when it has no row. A row that already exists is returned unchanged.
///
/// The Auth object runs one request at a time. `ON CONFLICT DO NOTHING` still keeps a repeated claim from replacing
/// a mapping that appeared before the insert.
///
/// # Arguments
/// * `db` - Database holding the `libraries` table.
/// * `account` - Account name from [`account_of`].
///
/// # Errors
/// An internal error when the account is empty or every candidate name is taken, or the database's error.
pub fn claim_library(db: &dyn Sql, account: &str) -> Result<String> {
    if account.is_empty() {
        return Err(Error::internal("account is empty"));
    }
    db.transaction(|| {
        if let Some(existing) = account_library(db, account)? {
            return Ok(existing);
        }
        let base = library_name(account);
        let mut name = base.clone();
        let mut suffix = 2u32;
        loop {
            let taken = db.one("SELECT account FROM libraries WHERE library = ?", params![&name])?.and_then(|row| row.text("account").map(str::to_owned));
            match taken.as_deref() {
                None => break,
                Some(owner) if owner == account => return Ok(name),
                Some(_) => {
                    name = format!("{base}-{suffix}");
                    suffix += 1;
                    if suffix > 1000 {
                        return Err(Error::internal("could not allocate a library name"));
                    }
                }
            }
        }
        let inserted = db.execute(
            "INSERT INTO libraries (account, library) VALUES (?, ?) ON CONFLICT (account) DO NOTHING",
            params![account, &name],
        )?;
        if inserted == 0 {
            return account_library(db, account)?.ok_or_else(|| Error::internal("library claim lost the account row"));
        }
        Ok(name)
    })
}

// The library row for an account, if one is stored.
fn account_library(db: &dyn Sql, account: &str) -> Result<Option<String>> {
    Ok(db.one("SELECT library FROM libraries WHERE account = ?", params![account])?.and_then(|row| row.text("library").map(str::to_owned)))
}

// Up to 32 lower-case letters, digits and single dashes from the account's email or username.
fn library_slug(account: &str) -> String {
    let rest = account.split_once(':').map(|(_, rest)| rest).unwrap_or(account);
    let mut out = String::new();
    for byte in rest.bytes() {
        let byte = byte.to_ascii_lowercase();
        let ch = if byte.is_ascii_lowercase() || byte.is_ascii_digit() { byte as char } else { '-' };
        if ch == '-' && (out.is_empty() || out.ends_with('-')) {
            continue;
        }
        out.push(ch);
        if out.len() == 32 {
            break;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}
