use crate::database::connection::get_db;
use crate::errors::auth::AuthError;
use crate::models::auth::LoginFormData;
use crate::models::user::{Identifier, User, UserIdentifier};
use crate::models::{
    auth::RegistrationFormData,
    user::CreateUser
};
use anyhow::{anyhow, Context, Result};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, PasswordVerifier},
};
use garde::Validate;
use rand::rngs::OsRng;
use surrealdb::RecordId;

pub async fn register_user(form: RegistrationFormData) -> Result<RecordId> {
    let db = get_db();

    form.validate()
        .map_err(AuthError::InvalidData)
        .with_context(|| "The form validation for registration failed")?;
    form.validate_uniqueness().await?;

    let password_bytes = form.password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password_bytes, &salt)
        .map_err(AuthError::PasswordHashError)?;
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
            .map_err(|e| AuthError::DatabaseError(Box::new(e)))
            .with_context(|| "Failed to successfully create a user with their identifier, the database Transaction failed")?;

        let created_user_option: Option<User> = result.take(0)
            .map_err(|e| AuthError::DatabaseError(Box::new(e)))?;
        let created_user: User = created_user_option.ok_or_else(|| anyhow!("User Creation returned no data"))?;
        let user_id = created_user.id;

    Ok(user_id)
}

pub async fn authenticate(form: LoginFormData) -> Result<RecordId> {
    let db = get_db();

    let (identifier_type, identifier_value) = match form.identifier {
        Identifier::Email(email) => ("email", email),
        Identifier::Mobile(mobile) => ("mobile", mobile),
    };

    let mut result = db.query("SELECT * FROM user_identifier WHERE identifier_type = $identifier_type AND identifier_value = $identifier_value")
        .bind(("identifier_type", identifier_type))
        .bind(("identifier_value", identifier_value))
        .await
        .map_err(|e| AuthError::DatabaseError(Box::new(e)))
        .with_context(|| "Failed to get search for the identifier for authentication")?;

    let user_identifier_option: Option<crate::models::user::UserIdentifier> = result.take(0)
        .map_err(|e| AuthError::DatabaseError(Box::new(e)))
        .with_context(|| "failed to get the result for the request user identifier")?;
    let user_identifier: UserIdentifier =
        user_identifier_option.ok_or(AuthError::UserNotFound)?;

    let mut user_result = db
        .query("SELECT * FROM user WHERE id = $user_id")
        .bind(("user_id", user_identifier.user_id))
        .await
        .map_err(|e| AuthError::DatabaseError(Box::new(e)))
        .with_context(|| "Failed to get user from user_id")?;

    let requested_user_option: Option<User> = user_result.take(0)
        .map_err(|e| AuthError::DatabaseError(Box::new(e)))
        .with_context(|| "failed to get the result for the request user")?;
    let requested_user: User = requested_user_option.ok_or(AuthError::UserNotFound)?;

    let parsed_hash = argon2::password_hash::PasswordHash::new(&requested_user.password_hash)
        .map_err(AuthError::PasswordHashError)?;

    let argon2 = Argon2::default();
    argon2.verify_password(form.password.as_bytes(), &parsed_hash)
        .map_err(AuthError::PasswordVerificationError)
        .with_context(|| "Password verification failed")?;

    Ok(requested_user.id)
}
