use actix_web::cookie::Cookie;
use actix_web::HttpRequest;
use actix_web::{delete, get, post, web, HttpResponse};
use anyhow::{anyhow, Result};
use base64::Engine as _; // base64: check out this classy github issue! https://github.com/marshallpierce/rust-base64/issues/213
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::io::Read;

use expiring_map::ExpiringMap;
use once_cell::sync::Lazy; // 1.3.1
use std::sync::Mutex;

use super::super::schema_generated::*;
use super::super::schema_types::*;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

use oauth2::{basic::BasicClient, TokenResponse};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, RedirectUrl, RevocationUrl, Scope, TokenUrl,
};
use std::env;

use crate::session;

// todo - session cookie first, then csrf storage and retreival in some kind of cache with expiration
// https://security.stackexchange.com/questions/20187/oauth2-cross-site-request-forgery-and-state-parameter

type GoogleClientType =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointSet, EndpointSet>;

fn get_google_client() -> GoogleClientType {
    let google_client_id = ClientId::new(env!("GOOGLE_CLIENT_ID").to_string());
    let google_client_secret = ClientSecret::new(env!("GOOGLE_CLIENT_SECRET").to_string());

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    // Set up the config for the Google OAuth2 process
    BasicClient::new(google_client_id)
        .set_client_secret(google_client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(
            RedirectUrl::new(format!("{}/api/authRedirect", env!("BACKEND_BASE")))
                .expect("Invalid redirect URL"),
        )
        // Google supports OAuth 2.0 Token Revocation (RFC-7009)
        .set_revocation_url(
            RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
                .expect("Invalid revocation endpoint URL"),
        )
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
struct AuthURLs {
    google: Option<String>,
}

#[get("/api/authURLs")]
async fn get_auth_urls(req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    println!("/authURLs");
    let (session_value, outgoing_cookie) = get_session_value(req, true);
    if session_value.is_none() {
        return Ok(HttpResponse::InternalServerError().finish());
    }

    let client = get_google_client();

    // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user
    let (google_auth_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // see https://developers.google.com/identity/protocols/oauth2/scopes#oauth2
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.email".to_string(), // todo - maybe tighten this up to only userinfo.email?
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    // save oauth info to verify the redirect query string that comes back
    session::insert_oauth_info(
        session_value.unwrap(),
        session::OAuthVerificationInfo {
            pkce_code_verifier: Some(pkce_code_verifier),
            csrf_state,
        },
    );

    if let Some(outgoing_cookie) = outgoing_cookie {
        println!("setting cookie: {:#?}", outgoing_cookie);
        Ok(HttpResponse::Ok().cookie(outgoing_cookie).json(AuthURLs {
            google: Some(google_auth_url.to_string()),
        }))
    } else {
        println!("not setting cookie");
        // todo - put google url under some json or something
        Ok(HttpResponse::Ok().json(AuthURLs {
            google: Some(google_auth_url.to_string()),
        }))
    }
}

#[derive(Debug, Deserialize)]
struct GoogleAuthQuery {
    state: Option<String>,
    code: Option<String>,
    // scope: Option<String>,
    // authuser: Option<String>,
    // prompt: Option<String>,
    // session_state: Option<String>,
    // hd: Option<String>,
}

// todo: cache
pub fn get_oauth_entry_db(
    db_conn: &mut SqliteConnection,
    unique_id: String,
) -> Result<UserOauthEntry, diesel::result::Error> {
    user_oauth_entries::dsl::user_oauth_entries
        .filter(user_oauth_entries::unique_id.eq(unique_id))
        .order(user_oauth_entries::id.desc())
        .first::<UserOauthEntry>(db_conn)
}

// todo: cache
pub fn get_user_db(
    db_conn: &mut SqliteConnection,
    user_id: i32,
) -> Result<User, diesel::result::Error> {
    users::dsl::users
        .filter(users::id.eq(user_id))
        .order(users::id.desc())
        .first::<User>(db_conn)
}

#[skip_serializing_none]
#[derive(Default, Serialize)]
pub struct FullUser {
    user: User,
    oauth: Vec<UserOauthEntry>,
    // sessions: Vec<UserSession>, // not sure I want to share this out to expose all session keys based on one session
}

pub fn get_user_and_oauth_db(
    db_conn: &mut SqliteConnection,
    user_id: i32,
) -> Result<FullUser, diesel::result::Error> {
    let user = users::dsl::users
        .filter(users::id.eq(user_id))
        .order(users::id.desc())
        .first::<User>(db_conn);

    match user {
        Ok(user) => {
            let oauth = user_oauth_entries::dsl::user_oauth_entries
                .filter(user_oauth_entries::user_id.eq(user_id))
                .load::<UserOauthEntry>(db_conn);

            let mut output = FullUser {
                user,
                ..Default::default()
            };

            if let Ok(oauth) = oauth {
                output.oauth = oauth;
            }

            Ok(output)
        }
        Err(e) => Err(e),
    }
}

#[derive(Clone)]
pub struct AccountOffer {
    used: bool,
    google_account_info: Option<GoogleAccountInfo>,
}

// if a user comes back from an external oauth redirect with a valid external account but no website account yet, offer to create a website account
// cache the offer for N minutes so that the user can hit the "ok, create" API
static ACCOUNT_OFFER_CACHE: Lazy<Mutex<ExpiringMap<String, AccountOffer>>> =
    Lazy::new(|| Mutex::new(ExpiringMap::new(std::time::Duration::from_secs(30 * 60))));

pub fn insert_account_offer(session_value: String, account_offer: AccountOffer) {
    ACCOUNT_OFFER_CACHE
        .lock()
        .unwrap()
        .insert(session_value, account_offer);
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct GoogleAccountInfo {
    id: String,           // won't change, primary key
    email: String,        // could change. store this anyway so we can greet the user correctly
    verified_email: bool, // require true
    picture: String,      // we don't use this
}

#[skip_serializing_none]
#[derive(Default, Serialize)]
pub struct ReceiveRedirectReturn {
    user: Option<User>, // give this back if we already have a matching user
    account_info: Option<GoogleAccountInfo>, // give this back if we don't have a user created yet (with account_offer = true)
    account_offer: bool,
}

fn receive_oauth_redirect_db(
    query: web::Query<GoogleAuthQuery>,
    session_value: String,
    db_conn: &mut SqliteConnection,
) -> Result<ReceiveRedirectReturn> {
    println!("{:?}", query);

    if query.code.is_none() || query.state.is_none() {
        return Err(anyhow!("missing query info"));
    }

    let code = AuthorizationCode::new(query.code.as_ref().unwrap().clone());
    let state = CsrfToken::new(query.state.as_ref().unwrap().clone());

    // will fail if we didn't have an existing oauth request that matches this redirect
    let oauth_info = session::get_oauth_info(&session_value);

    if oauth_info.is_err() {
        return Err(anyhow!("missing oauth info"));
    }

    let (pkce_code_verifier, csrf_state) = oauth_info.unwrap();

    println!(
        "oauth redirect returned the following code:\n{}\n",
        code.secret()
    );
    println!(
        "oauth redirect returned the following state:\n{} (expected `{}`)\n",
        state.secret(),
        csrf_state
    );

    let client = get_google_client();
    let http_client = oauth2::reqwest::blocking::ClientBuilder::new()
        .redirect(oauth2::reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let token_response = client
        .exchange_code(code)
        .set_pkce_verifier(pkce_code_verifier)
        .request(&http_client);

    println!("token response: {:#?}", token_response);
    let token_secret = token_response.unwrap().access_token().secret().clone();

    println!("token: {:?}", token_secret);

    let mut resp = http_client
        .get(format!(
            "https://www.googleapis.com/oauth2/v1/userinfo?access_token={}",
            token_secret
        ))
        .send()?;

    let mut body = String::new();
    let _ = resp.read_to_string(&mut body); // ignore errors, we'll catch them in json parsing
    println!("body: {:?}", body);
    let account_info: GoogleAccountInfo = serde_json::from_str(&body)?;

    println!("{:#?} body: {:#?}", resp, account_info);

    // todo -
    // - if we already have a user, associate this session to that user (database)
    // if account_info.id is in user_oauth_entries
    if let Ok(oauth_entry) = get_oauth_entry_db(db_conn, format!("google:{}", account_info.id)) {
        if let Ok(user) = get_user_db(db_conn, oauth_entry.user_id) {
            session::store_session(
                db_conn,
                UserSessionToInsert {
                    user_id: user.id,
                    session_value,
                    created: chrono::Utc::now().timestamp(),
                },
            );
            return Ok(ReceiveRedirectReturn {
                user: Some(user),
                account_info: None,
                account_offer: false,
            });
        } else {
            // todo - have oauth entry but no user, should delete the oauth entry or print something
        }
    }

    // - if we don't have a user, put them into an account offer pool. timeout like 30 minutes (another singleton map)
    insert_account_offer(
        session_value,
        AccountOffer {
            google_account_info: Some(account_info.clone()),
            used: false,
        },
    );
    Ok(ReceiveRedirectReturn {
        user: None,
        account_info: Some(account_info),
        account_offer: true,
    })
}

pub fn get_session_value(
    req: HttpRequest,
    set_session: bool,
) -> (Option<String>, Option<Cookie<'static>>) {
    let incoming_cookie = req.cookie("session");
    let outgoing_cookie: Option<Cookie>;

    let session_value: Option<String>;
    if let Some(incoming_cookie) = incoming_cookie {
        println!("existing session value: {:#?}", incoming_cookie.value());

        // todo - make sure the existing session is the right length? want to prevent users from making their own session token. or else, overwrite it when logging in or otherwise starting some auth thing with it

        session_value = Some(incoming_cookie.value().to_string());
        outgoing_cookie = None;
    } else if set_session {
        println!("no session value found, setting");
        // set a random session
        session_value = Some(
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(rand::random::<[u8; 32]>()),
        );

        outgoing_cookie = Some(
            Cookie::build("session", session_value.as_ref().unwrap().clone())
                .domain(env!("COOKIE_DOMAIN"))
                .path("/")
                //  .same_site(actix_web::cookie::SameSite::Strict)
                //  .secure(true)
                .http_only(true)
                .finish(),
        );
    } else {
        println!("no session value found, not setting");
        session_value = None;
        outgoing_cookie = None;
    }

    (session_value, outgoing_cookie)
}

#[get("/api/authRedirect")]
async fn receive_oauth_redirect(
    req: HttpRequest,
    query: web::Query<GoogleAuthQuery>,
    pool: web::Data<DbPool>,
) -> actix_web::Result<impl actix_web::Responder> {
    let (session_value, _outgoing_cookie) = get_session_value(req, false);

    if session_value.is_none() {
        return Err(actix_web::error::ErrorInternalServerError(""));
    }

    let results = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        receive_oauth_redirect_db(query, session_value.unwrap(), &mut conn)
    })
    .await
    .unwrap();

    let results = match results {
        Ok(results) => results,
        Err(e) => {
            eprintln!("{}", e);
            return Err(actix_web::error::ErrorInternalServerError(e));
        }
    };

    // redirect to a post-login page (either account offer or logged-in landing page)
    // todo: encode our account info somewhere, I guess in a query string in the redirect?
    if results.account_offer {
        Ok(HttpResponse::Found()
            .append_header(("Location", env!("CREATE_ACCOUNT_REDIRECT")))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .append_header(("Location", env!("AUTHED_REDIRECT")))
            .finish())
    }
}

#[skip_serializing_none]
#[derive(Default, Deserialize, Serialize, Clone)]
pub struct UserCreateQuery {
    pub name: Option<String>,
    pub check_only: Option<bool>, // if true, we're only looking for name conflicts in order to show a quick "available" note when registering
}

pub fn create_account_db(
    session_value: String,
    query: &UserCreateQuery,
    db_conn: &mut SqliteConnection,
) -> Result<Option<FullUser>> {
    if let Some(offer) = ACCOUNT_OFFER_CACHE.lock().unwrap().get_mut(&session_value) {
        if offer.used {
            return Err(anyhow!("account offer already used"));
        }

        if query.name.is_none() {
            return Err(anyhow!("account name missing"));
        }
        let name = query.name.clone().unwrap();

        // todo lint username: min length, prevent all-whitespace or all-punctuation. trim leading and trailing whitespace. etc.
        // todo - store a lower cased and de-punctuated version and de-dupe that too
        // todo - rate limit username availability check

        // todo - this would need to skip the offer check (and require a valid session I guess)
        // in order to allow using it to check names while changing usernames
        if let Some(check_only) = query.check_only {
            if check_only {
                let existing_user = users::dsl::users
                    .filter(users::name.eq(name.clone()))
                    .order(users::id.desc())
                    .first::<User>(db_conn);

                return match existing_user {
                    Ok(_) => Err(anyhow!("username taken")),
                    Err(_error) => Ok(None),
                };
            }
        }

        // we're good to create the account, mark the offer used
        offer.used = true;

        let google_account = offer.google_account_info.as_ref().unwrap();
        let email = google_account.email.clone();
        let rows_inserted = diesel::insert_into(users::dsl::users)
            .values((users::name.eq(name.clone()), users::email.eq(email.clone())))
            .execute(db_conn);

        if rows_inserted != Ok(1) {
            return Err(anyhow!("couldn't create account"));
        }

        // get the id of the newly-created user (sqlite can't return this from the creation query)
        let new_user = users::dsl::users
            .filter(users::name.eq(name))
            .order(users::id.desc())
            .first::<User>(db_conn)?;

        let rows_inserted = diesel::insert_into(user_oauth_entries::dsl::user_oauth_entries)
            .values((
                user_oauth_entries::user_id.eq(new_user.id),
                user_oauth_entries::unique_id.eq(format!("google:{}", google_account.id)),
                user_oauth_entries::oauth_info.eq(serde_json::to_string(&google_account)?), // save this for display purposes or whatever
            ))
            .execute(db_conn);

        if rows_inserted != Ok(1) {
            return Err(anyhow!("couldn't create oauth entry")); // todo - a transaction to cover account + oauth entry creation
        }

        // set the user logged in
        session::store_session(
            db_conn,
            UserSessionToInsert {
                user_id: new_user.id,
                session_value,
                created: chrono::Utc::now().timestamp(),
            },
        );
        // return the user
        match get_user_and_oauth_db(db_conn, new_user.id) {
            Ok(fulluser) => Ok(Some(fulluser)),
            Err(_e) => {
                Err(anyhow!("error getting account after creation")) // todo maybe convert the error?
            }
        }
    } else {
        // no offer in the cache
        Err(anyhow!("no account offer in cache"))
    }
}

#[get("/api/createAccount")]
async fn create_account(
    req: HttpRequest,
    query: web::Query<UserCreateQuery>,
    pool: web::Data<DbPool>,
) -> actix_web::Result<impl actix_web::Responder> {
    // todo - user gets to fill in other fields like nickname or whatever, maybe in the query string

    // look at account offer cache for this session
    let (session_value, _outgoing_cookie) = get_session_value(req, false);
    if session_value.is_none() {
        return Ok(HttpResponse::InternalServerError().finish());
    }

    let result = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        create_account_db(session_value.unwrap(), &query, &mut conn)
    })
    .await
    .unwrap(); // todo - blockingerror unwrap?

    match result {
        Ok(results) => Ok(HttpResponse::Ok().json(results)),
        Err(e) => {
            eprintln!("{}", e);
            Err(actix_web::error::ErrorInternalServerError(e))
        }
    }
}

#[get("/api/user")]
async fn get_user(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let (session_value, _outgoing_cookie) = get_session_value(req, false);
    if session_value.is_none() {
        return Ok(HttpResponse::InternalServerError().finish());
    }

    let mut db_conn = pool.get().expect("couldn't get db connection from pool");

    let session = session::get_session(&mut db_conn, session_value.unwrap());
    if session.is_err() {
        return Ok(HttpResponse::NotFound().finish());
    }
    let session = session.unwrap();

    let info = get_user_and_oauth_db(&mut db_conn, session.user_id);
    if info.is_err() {
        return Ok(HttpResponse::NotFound().finish());
    }

    Ok(HttpResponse::Ok().json(info.unwrap()))
}

#[get("/api/checkLogin")]
async fn check_login(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let (session_value, _outgoing_cookie) = get_session_value(req, false);
    if session_value.is_none() {
        return Ok(HttpResponse::InternalServerError().finish());
    }

    let mut db_conn = pool.get().expect("couldn't get db connection from pool");

    let session = session::get_session(&mut db_conn, session_value.unwrap());
    if session.is_err() {
        return Ok(HttpResponse::NotFound().finish());
    }
    let session = session.unwrap();

    let user = get_user_db(&mut db_conn, session.user_id);
    if user.is_err() {
        return Ok(HttpResponse::NotFound().finish());
    }

    // return account if this session is logged in. some error otherwise
    Ok(HttpResponse::Ok().json(user.unwrap()))
}

fn get_logout_cookie() -> Cookie<'static> {
    Cookie::build("session", "")
        .domain(env!("COOKIE_DOMAIN"))
        .path("/")
        //  .same_site(actix_web::cookie::SameSite::Strict)
        //  .secure(true)
        .expires(actix_web::cookie::Expiration::from(
            actix_web::cookie::time::OffsetDateTime::now_utc(),
        )) // time in the past clears a cookie
        .http_only(true)
        .finish()
}

#[post("/api/logout")]
async fn logout(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let (session_value, _outgoing_cookie) = get_session_value(req, false);
    if session_value.is_none() {
        eprintln!("post /api/logout called without a session");
        return Ok(HttpResponse::InternalServerError().finish());
    }

    let mut db_conn = pool.get().expect("couldn't get db connection from pool");

    session::remove_session(&mut db_conn, session_value.unwrap());
    let outgoing_cookie = get_logout_cookie();
    Ok(HttpResponse::Ok().cookie(outgoing_cookie).json(""))
}

#[delete("/api/user")]
async fn delete_user(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let (session_value, _outgoing_cookie) = get_session_value(req, false);
    if session_value.is_none() {
        return Ok(HttpResponse::InternalServerError().finish());
    }

    let mut db_conn = pool.get().expect("couldn't get db connection from pool");

    let session = session::get_session(&mut db_conn, session_value.clone().unwrap());
    if session.is_err() {
        return Ok(HttpResponse::NotFound().finish());
    }
    let session = session.unwrap();

    let db_result = db_conn.exclusive_transaction::<_, diesel::result::Error, _>(|db_conn| {
        // todo - a transaction to cover user+oauth+list deletion
        // add a transaction to the create function also
        if let Err(_) =
            diesel::delete(users::dsl::users.filter(users::id.eq(session.user_id))).execute(db_conn)
        {
            return Ok(HttpResponse::NotFound().finish());
        }

        let _ = diesel::delete(
            user_oauth_entries::dsl::user_oauth_entries
                .filter(user_oauth_entries::user_id.eq(session.user_id)),
        )
        .execute(db_conn);

        // user lists
        let _ = diesel::delete(
            locations::dsl::locations.filter(locations::user_id.eq(session.user_id)),
        )
        .execute(db_conn);

        session::remove_session(db_conn, session_value.unwrap()); // todo - delete cached sessions, if implemented

        Ok(HttpResponse::Ok().finish())
    });

    if let Err(e) = db_result {
        return Err(actix_web::error::ErrorInternalServerError(e));
    }

    let outgoing_cookie = get_logout_cookie();
    Ok(HttpResponse::Ok().cookie(outgoing_cookie).finish())
}
