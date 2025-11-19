use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("The form data provided is invalid")]
    InvalidData(#[from] garde::Report),

    #[error("Database operation failed")]
    DatabaseError(#[from] Box<surrealdb::Error>),

    #[error("{0} already registered")]
    NotUniqueError(String),

    #[error("Failed to hash the password")]
    PasswordHashError(argon2::password_hash::Error),

    #[error("Password verification failed")]
    PasswordVerificationError(argon2::password_hash::Error),

    #[error("Requested user was not found")]
    UserNotFound,
}

