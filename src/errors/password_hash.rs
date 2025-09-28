use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct PasswordHashError(argon2::password_hash::Error);

impl From<argon2::password_hash::Error> for PasswordHashError {
    fn from(err: argon2::password_hash::Error) -> Self {
        PasswordHashError(err)
    }
}
