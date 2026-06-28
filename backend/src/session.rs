use diesel::SqliteConnection;
// store session data in memory. could be refactored to use redis or something if we went to multiple servers
use expiring_map::ExpiringMap;
use once_cell::sync::Lazy; // 1.3.1
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{anyhow, Result};
use oauth2::{CsrfToken, PkceCodeVerifier};

use super::schema_generated::*;
use super::schema_types::*;
use diesel::prelude::*;

// holds info we need to keep after sending a user off to an external oauth provider, for verification of the returned data
#[derive(Debug)]
pub struct OAuthVerificationInfo {
    pub pkce_code_verifier: Option<PkceCodeVerifier>, // Option<> because the library author is our mom and doesn't want us reusing this
    pub csrf_state: CsrfToken,
}

static OAUTH_INFO_CACHE: Lazy<Mutex<ExpiringMap<String, OAuthVerificationInfo>>> =
    Lazy::new(|| Mutex::new(ExpiringMap::new(Duration::from_secs(60))));

pub fn insert_oauth_info(session_value: String, oauth_info: OAuthVerificationInfo) {
    println!("insert oauth info for {}", session_value);

    OAUTH_INFO_CACHE
        .lock()
        .unwrap()
        .insert(session_value, oauth_info);
}

pub fn get_oauth_info(session_value: &str) -> Result<(PkceCodeVerifier, String)> {
    let pkce_code_verifier;
    let csrf_state;

    println!("get oauth info for {}", session_value);

    // get these things out of OAUTH_INFO so we can drop the lock right away
    if let Some(oauth_info) = OAUTH_INFO_CACHE.lock().unwrap().get_mut(session_value) {
        // get ownership out of the Option<>
        let pkce_code_verifier_option = oauth_info.pkce_code_verifier.take();
        csrf_state = oauth_info.csrf_state.secret().clone();

        if pkce_code_verifier_option.is_none() {
            return Err(anyhow!("oauth info already used"));
        }
        pkce_code_verifier = pkce_code_verifier_option.unwrap();
    } else {
        return Err(anyhow!("oauth info not found"));
    }

    Ok((pkce_code_verifier, csrf_state))
}

// todo: session load/store and cache

static SESSION_CACHE: Lazy<Mutex<ExpiringMap<String, UserSession>>> =
    Lazy::new(|| Mutex::new(ExpiringMap::new(Duration::from_secs(7 * 24 * 60 * 60))));

pub fn get_session(db_conn: &mut SqliteConnection, session_value: String) -> Result<UserSession> {
    // first look in our cache
    if let Some(cache_return) = SESSION_CACHE.lock().unwrap().get(&session_value) {
        return Ok(cache_return.clone());
    }

    // if not in the cache, it could be in the database (should only happen if the server was restarted)
    // if we loaded the database into the cache on program start then this could be omitted
    let db_return: Result<UserSession, diesel::result::Error> = user_sessions::dsl::user_sessions
        .filter(user_sessions::session_value.eq(session_value))
        .first(db_conn);

    match db_return {
        Ok(db_return) => {
            // todo: check expiration time on the database item
            Ok(db_return)
        }
        Err(error) => Err(error.into()),
    }
}

pub fn store_session(db_conn: &mut SqliteConnection, session: UserSessionToInsert) {
    // store in cache and also in our database
    SESSION_CACHE.lock().unwrap().insert(
        session.session_value.clone(),
        UserSession {
            id: 0, // fake database row id, we don't use it
            user_id: session.user_id,
            session_value: session.session_value.clone(),
            created: session.created,
        },
    );

    // store in the database too
    let result = diesel::insert_into(user_sessions::dsl::user_sessions)
        .values((
            user_sessions::user_id.eq(session.user_id),
            user_sessions::session_value.eq(session.session_value),
            user_sessions::created.eq(session.created),
        ))
        .execute(db_conn);

    if result != Ok(1) {
        println!("failed adding session to the database")
    }

    // todo: every so often, delete old sessions from the database (sqlite doesn't have TTL like redis does)
}

pub fn remove_session(db_conn: &mut SqliteConnection, session_value: String) {
    let _deleted = diesel::delete(
        user_sessions::dsl::user_sessions
            .filter(user_sessions::session_value.eq(session_value.clone())),
    )
    .execute(db_conn);
    // todo - error if we didn't find anything?
    {
        SESSION_CACHE.lock().unwrap().remove(session_value);
    }
}
