use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistrationError {
    #[error("The form data provided is invalid")]
    InvalidData(#[from] garde::Report),

    #[error("Database operation failed")]
    DatabaseError(#[from] Box<surrealdb::Error>),

    #[error("{0} already registered")]
    NotUniqueError(String),

    #[error("Failed to hash the password")]
    PasswordHashError(argon2::password_hash::Error)
}

impl From<argon2::password_hash::Error> for RegistrationError {
    fn from(err: argon2::password_hash::Error) -> Self {
        RegistrationError::PasswordHashError(err)
    }
}

