use crate::database::connection::get_db;
use crate::errors::registration::RegistrationError;
use crate::models::{
    auth::RegistrationFormData,
    user::CreateUser
};
use anyhow::{Context, Result};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};
use garde::Validate;
use rand::rngs::OsRng;
use surrealdb::{RecordId, Uuid};

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

    let unique_id = Uuid::new_v4().to_string();
    let user_id = RecordId::from(("users", unique_id));

    let user = CreateUser {
        id: user_id.clone(),
        display_name: form.name,
        password_hash: password_hash_str,
    };

    let identifier_data = form.identifier;

    let surql = r#"
            BEGIN TRANSACTION;

            LET $created_user = (CREATE users CONTENT $user_data)[0];

            CREATE user_identifier CONTENT {
                user_id: $created_user.id,
                identifier_type: $identifier_data.identifier_type,
                identifier_value: $identifier_data.indentifier_value
            };

            COMMIT TRANSACTION; 
        "#;

        db.query(surql)
            .bind(("user_data", user))
            .bind(("identifier_data", identifier_data))
            .await
            .map_err(|e| RegistrationError::DatabaseError(Box::new(e)))
            .with_context(|| "Failed to successfully create a user with their identifier, the database transaction failed")?;

    Ok(user_id)
}
