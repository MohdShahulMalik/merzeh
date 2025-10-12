use actix_web::http::header::{HeaderValue, SET_COOKIE};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use leptos::prelude::expect_context;
use leptos_actix::ResponseOptions;
use surrealdb::RecordId;
use surrealdb::sql::Datetime;

use crate::{
    database::connection::get_db,
    errors::session::SessionError,
    models::{
        session::{CreateSession, Session, UpdateSession},
        user::User,
    },
    utils::token_generator::generate_token,
};

static SESSION_DURATION_IN_HOURS: i64 = 1;

pub async fn create_session(user_id: RecordId) -> Result<String> {
    let db = get_db();

    let session_token = generate_token();
    let expires_at = Datetime::from(Utc::now() + Duration::hours(SESSION_DURATION_IN_HOURS));

    let session = CreateSession {
        user_id,
        session_token: session_token.clone(),
        expires_at,
    };

    let _: Option<CreateSession> = db
        .create("sessions")
        .content(session)
        .await
        .map_err(|e| SessionError::DatabaseError(Box::new(e)))
        .with_context(|| "Failed to create a session")?;

    Ok(session_token)
}

pub async fn get_user_by_session(session_token: &str) -> Result<User> {
    validate_session_token(session_token)?;

    let db = get_db();

    let result_from_sessions_table: Option<Session> = db
        .query("SELECT * FROM sessions WHERE session_token = $token")
        .bind(("token", session_token.to_string()))
        .await
        .map_err(|e| SessionError::DatabaseError(Box::new(e)))
        .with_context(|| "Failed to fetch the session details")?
        .take(0)?;

    if let Some(session) = result_from_sessions_table {
        if session.expires_at >= Datetime::from(Utc::now()) {
            Err(SessionError::SessionExpired(session.expires_at))?;
        }

        let result_from_user_table = db.select(session.user_id).await?;

        if let Some(user) = result_from_user_table {
            Ok(user)
        } else {
            Err(SessionError::UserNotFound)?
        }
    } else {
        Err(SessionError::SessionNotFound)?
    }
}

pub async fn delete_session(session_token: &str) -> Result<()> {
    validate_session_token(session_token)?;

    let db = get_db();

    db.query("DELETE sessions WHERE session_token = $token")
        .bind(("token", session_token.to_string()))
        .await
        .map_err(|e| SessionError::DatabaseError(Box::new(e)))
        .with_context(|| "Failed to delete the session ")?;

    Ok(())
}

pub async fn update_session_token(user_id: RecordId) -> Result<String> {
    let db = get_db();
    let new_session_token = generate_token();

    let updated_session = UpdateSession {
        session_token: Some(new_session_token.clone()),
        expires_at: None,
    };

    let _: Option<Session> = db
        .update(user_id.clone())
        .merge(updated_session)
        .await
        .map_err(|e| SessionError::DatabaseError(Box::new(e)))
        .with_context(|| "Failed to update the token for a user")?;

    Ok(new_session_token)
}

pub async fn update_session_expiry(user_id: RecordId) -> Result<()> {
    let db = get_db();

    let session: Option<Session> = db
        .select(user_id.clone())
        .await
        .map_err(|e| SessionError::DatabaseError(Box::new(e)))
        .with_context(|| "Failed to fetch session for it to update")?;

    let session = session.ok_or(SessionError::SessionNotFound)?;
    let old_expired_at: chrono::DateTime<Utc> = session.expires_at.into();
    let new_expired_at =
        Datetime::from(old_expired_at + Duration::hours(SESSION_DURATION_IN_HOURS));

    let updated_session = UpdateSession {
        session_token: None,
        expires_at: Some(new_expired_at),
    };

    let _: Option<Session> = db
        .update(user_id)
        .merge(updated_session)
        .await
        .map_err(|e| SessionError::DatabaseError(Box::new(e)))
        .with_context(|| "Failed to fetch session record to update the expiry time")?;

    Ok(())
}

pub async fn update_session_expiry_and_token(user_id: RecordId) -> Result<String> {
    let db = get_db();

    let session: Option<Session> = db
        .select(user_id.clone())
        .await
        .map_err(|e| SessionError::DatabaseError(Box::new(e)))
        .with_context(
            || "Failed to fetch the session to update its session token and expiry time",
        )?;

    let session = session.ok_or(SessionError::SessionNotFound)?;

    let old_expired_at: chrono::DateTime<Utc> = session.expires_at.into();
    let new_expired_at =
        Datetime::from(old_expired_at + Duration::hours(SESSION_DURATION_IN_HOURS));
    let new_session_token = generate_token();

    let updated_session = UpdateSession {
        session_token: Some(new_session_token.clone()),
        expires_at: Some(new_expired_at),
    };

    let _: Option<Session> = db
        .update(user_id)
        .merge(updated_session)
        .await
        .map_err(|e| SessionError::DatabaseError(Box::new(e)))
        .with_context(|| "Failed to update session's token and expiry time")?;

    Ok(new_session_token)
}

pub async fn cleanup_expired_sessions() -> Result<()> {
    let db = get_db();

    db.query("DELETE sessions WHERE expired_at <= time::now()")
        .await
        .map_err(|e| SessionError::DatabaseError(Box::new(e)))
        .with_context(|| "Failed to deleted expired sessions")?;

    Ok(())
}

pub fn set_session_cookie(
    session_token: &str
) -> Result<()> {
    
    let response = expect_context::<ResponseOptions>();

    let cookie = format!(
        "__Host-session={}; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age={}",
        session_token,
        SESSION_DURATION_IN_HOURS * 60 * 60
    );

    response.insert_header(
        SET_COOKIE,
        HeaderValue::from_str(&cookie)
            .with_context(|| "Failed to set sesion headers")?
    );

    Ok(())
}

pub fn validate_session_token(token: &str) -> Result<(), SessionError> {
    if token.is_empty() {
        Err(SessionError::InvalidToken)?
    }

    if token.len() < 40 || token.len() > 50 {
        Err(SessionError::InvalidToken)?
    }

    if !token
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        Err(SessionError::InvalidToken)?
    }

    Ok(())
}
