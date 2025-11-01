use crate::database::connection::get_db;
use crate::errors::registration::RegistrationError;
use crate::models::user::User;
use crate::models::{
    auth::RegistrationFormData,
    user::CreateUser
};
use anyhow::{anyhow, Context, Result};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};
use garde::Validate;
use rand::rngs::OsRng;
use surrealdb::RecordId;

pub async fn register_user(form: RegistrationFormData) -> Result<RecordId> {
    let db = get_db();

    form.validate()
        .map_err(RegistrationError::InvalidData)
        .with_context(|| "The form validation for registration failed")?;
    form.validate_uniqueness().await?;

    let password_bytes = form.password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password_bytes, &salt)
        .map_err(RegistrationError::from)?;
    let password_hash_str = password_hash.to_string();

    let user = CreateUser {
        display_name: form.name,
        password_hash: password_hash_str,
    };

    let identifier_data = form.identifier;

    let surql = r#"
            BEGIN TRANSACTION;

            LET $created_user = CREATE ONLY users CONTENT $user_data;

            CREATE user_identifier CONTENT {
                user_id: $created_user.id,
                identifier_type: $identifier_data.identifier_type,
                identifier_value: $identifier_data.identifier_value
            };

            RETURN $created_user;
            COMMIT TRANSACTION; 
        "#;

        let mut result = db.query(surql)
            .bind(("user_data", user))
            .bind(("identifier_data", identifier_data))
            .await
            .map_err(|e| RegistrationError::DatabaseError(Box::new(e)))
            .with_context(|| "Failed to successfully create a user with their identifier, the database Transaction failed")?;

        let created_user_option: Option<User> = result.take(0)
            .map_err(|e| RegistrationError::DatabaseError(Box::new(e)))?;
        let created_user: User = created_user_option.ok_or_else(|| anyhow!("User Creation returned no data"))?;
        let user_id = created_user.id;

    Ok(user_id)
}
