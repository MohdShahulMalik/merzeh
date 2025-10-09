use surrealdb::sql::Datetime;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session has been expired at: {0}")]
    SessionExpired(Datetime),

    #[error("Session Token Specified Not Found")]
    SessionNotFound,

    #[error("Invalid Session Token Format")]
    InvalidToken,

    #[error("Database error: {0}")]
    DatabaseError(#[from] Box<surrealdb::Error>),

    #[error("User not found for the session")]
    UserNotFound,
}
