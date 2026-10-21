//! The first user row of an empty library, so a new account can save a card on its first request.

use mm_js::json;
use sha2::{Digest, Sha256};

use crate::auth_store::{Identity, account_of};
use crate::sql::SqlExt;
use crate::store::Store;
use crate::{Error, Result, obj};

const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Name and email copied onto the user row of a library created at first sign-in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberProfile {
    /// Account key, `google:<email>` or `password:<username>`.
    pub account: String,
    /// Email, or the password username when the account has no email.
    pub email: String,
    /// Short name taken from the email or username.
    pub name: String,
}

/// Profile stored for a session. Google uses the email; a password session uses its `password:` subject.
///
/// # Arguments
/// * `sub` - Session subject.
/// * `email` - Email stored with the session.
pub fn profile_from_session(sub: &str, email: &str) -> MemberProfile {
    let identity = Identity {
        sub: sub.to_owned(),
        email: email.to_owned(),
    };
    let account = account_of(&identity);
    let email = if identity.email.is_empty() {
        account.clone()
    } else {
        mm_js::text::to_lower(&identity.email)
    };
    let name = display_name(&account, &email);
    MemberProfile {
        account,
        email,
        name,
    }
}

/// Insert the library's user when the store has none. A store that already has a user is left as it is.
///
/// # Arguments
/// * `store` - Library store whose schema is already installed.
/// * `profile` - Name and email for the new user.
///
/// # Returns
/// `true` when a user row was inserted.
///
/// # Errors
/// The database's error.
pub fn ensure_member(store: &Store, profile: &MemberProfile) -> Result<bool> {
    store.db().transaction(|| {
        if store.db().one("SELECT id FROM users LIMIT 1", &[])?.is_some() {
            return Ok(false);
        }
        let (id, account_uid, id_prefix) = member_ids(&profile.account);
        let settings = default_settings(profile)?;
        store.db().execute(
            "INSERT INTO users (id, account_uid, id_prefix, flags, settings) VALUES (?, ?, ?, 0, ?)",
            &[id.into(), account_uid.into(), id_prefix.into(), settings.into()],
        )?;
        Ok(true)
    })
}

// Local part of an email, or the password username.
fn display_name(account: &str, email: &str) -> String {
    let source = if let Some(username) = account.strip_prefix("password:") {
        username
    } else if let Some((local, _)) = email.split_once('@') {
        local
    } else if !email.is_empty() {
        email
    } else {
        account
    };
    if source.is_empty() {
        "Library".to_owned()
    } else {
        source.to_owned()
    }
}

// Stable ids. Card ids are the 11-character prefix, a 2-character kind and a 9-character counter.
fn member_ids(account: &str) -> (String, String, String) {
    (
        base62("id", account, 22),
        base62("uid", account, 22),
        base62("prefix", account, 11),
    )
}

fn base62(label: &str, account: &str, len: usize) -> String {
    let digest = Sha256::digest(format!("{label}\0{account}").as_bytes());
    (0..len)
        .map(|index| char::from(BASE62[digest[index] as usize % 62]))
        .collect()
}

fn default_settings(profile: &MemberProfile) -> Result<String> {
    let value = obj! {
        "name" => profile.name.as_str(),
        "email" => profile.email.as_str(),
        "colorScheme" => "system",
        "locale" => "en-US",
        "timeZoneId" => "UTC",
        "currency" => "USD",
        "aiLevel" => 3.0,
        "onboardFlags" => 0.0,
        "objectLimit" => 50_000.0,
    };
    json::stringify(&value.into())?.ok_or_else(|| Error::internal("member settings"))
}
