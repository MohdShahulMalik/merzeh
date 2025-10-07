use crate::models::{registration::FormData, user::{CreateUser, CreateUserIdentifier, User, UserIdentifier}};
use garde::Validate;
use anyhow::{Result, anyhow};
use argon2::{password_hash::{PasswordHasher, SaltString}, Argon2};
use rand::rngs::OsRng;
use crate::errors::password_hash::PasswordHashError;
use crate::database::connection::get_db;

pub async fn register(form: FormData) -> Result<()>{
    let db = get_db();
    form.validate()?;
    form.validate_uniqueness(&db).await?;
    let password_bytes = form.password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password_bytes, &salt).map_err(PasswordHashError::from)?;
    let password_hash_str = password_hash.to_string();

    let user = CreateUser {
        display_name: form.name,
        password_hash: password_hash_str,
    };

    let created_user: Option<User> = db.create("users").content(user).await?;

    let created_user = created_user.ok_or_else(|| anyhow!("Failed to create user record"))?;

    let user_identifier = CreateUserIdentifier {
        identifier: form.identifier,
        user_id: created_user.id,
    };

    let _: Option<UserIdentifier> = db.create("user_identifier").content(user_identifier).await?;

    Ok(())
}
