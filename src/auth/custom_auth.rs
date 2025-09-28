use crate::models::registration::FormData;
use anyhow::Result;
use argon2::{password_hash::{PasswordHasher, SaltString}, Argon2, PasswordHash};
use rand::rngs::OsRng;
use crate::errors::password_hash::PasswordHashError;

pub fn register(form: FormData) -> Result<()>{ 
    let password_bytes = form.password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password_bytes, &salt).map_err(PasswordHashError::from)?;
    Ok(())
}
