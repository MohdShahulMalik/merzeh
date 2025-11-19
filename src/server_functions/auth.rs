#[cfg(feature = "ssr")]
use actix_web::http::StatusCode;
#[cfg(feature = "ssr")]
use garde::Validate;
#[cfg(feature = "ssr")]
use leptos::prelude::expect_context;
use leptos::prelude::ServerFnError;
use leptos::*;
#[cfg(feature = "ssr")]
use leptos_actix::ResponseOptions;
#[cfg(feature = "ssr")]
use tracing::error;

#[cfg(feature = "ssr")]
use crate::auth::custom_auth::authenticate;
#[cfg(feature = "ssr")]
use crate::auth::session::{create_session, set_session_cookie};
#[cfg(feature = "ssr")]
use crate::auth::custom_auth::register_user;
#[cfg(feature = "ssr")]
use crate::errors::auth::AuthError;
use crate::models::auth::LoginFormData;
use crate::models::{api_responses::ApiResponse, auth::RegistrationFormData};

#[server(prefix = "/auth", endpoint = "register")]
pub async fn register(form: RegistrationFormData) -> Result<ApiResponse<String>, ServerFnError>{

    let response_option = expect_context::<ResponseOptions>();

    let validation_result = form.validate();

    if let Err(error) = validation_result {
        let errors = error
            .iter()
            .map(|(field, msg)| format!("{}, {}", field, msg))
            .collect::<Vec<_>>();
        error!(?errors);
        response_option.set_status(StatusCode::UNPROCESSABLE_ENTITY);
        return Ok(ApiResponse { data: None, error: Some(errors.join("\n"))})
    }

    let validation_result_for_uniqueness = form.validate_uniqueness().await;
    if let Err(error) = validation_result_for_uniqueness {
        error!(?error);
        response_option.set_status(StatusCode::CONFLICT);
        return Ok(ApiResponse { data: None, error: Some(format!("{}", error))});
    } 

    let registration_result = register_user(form).await;

    if let Err(error) = registration_result {
        error!(?error, "Failed to register the user");  
        return Err(ServerFnError::ServerError("Failed to register the user".to_string()));
    };

    let user_id = registration_result.ok();
    let session_creation_result = create_session(user_id.unwrap()).await;
    if let Err(error) = session_creation_result {
        error!(?error);
        return Err(ServerFnError::ServerError("Failed to generate session tokens for the registered user".to_string()));
    }

    let session_token = session_creation_result.ok().unwrap();
    let cookie_creation_result = set_session_cookie(&session_token);

    if let Err(error) = cookie_creation_result {
        error!(?error);
        return Err(ServerFnError::ServerError("Failed to create appropriate cookies after registration".to_string()));
    }

    Ok(ApiResponse {
        data: Some("The user have been registered successfully".to_string()),
        error: None,
    })
}

#[server]
pub async fn login(
    form: LoginFormData,
) -> Result<ApiResponse<String>, ServerFnError>{
    let response_option = expect_context::<ResponseOptions>();
    let user_id = match authenticate(form).await {
        Ok(id) => id,
        Err(error) => {
            if let Some(auth_error) = error.downcast_ref::<AuthError>() {
                match auth_error {
                    AuthError::UserNotFound | AuthError::PasswordVerificationError(_) => {
                        error!("Authentication failed for user.");
                        response_option.set_status(StatusCode::UNAUTHORIZED);
                        return Ok(ApiResponse { data: None, error: Some("Invalid username or password.".to_string())});
                    },
                    AuthError::DatabaseError(_) | AuthError::PasswordHashError(_) => {
                        error!(?error, "Internal server error during authentication.");
                        response_option.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                        return Ok(ApiResponse { data: None, error: Some("An internal error occurred.".to_string())});
                    },
                    _ => {
                        error!(?error, "An unexpected authentication error occurred.");
                        response_option.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                        return Ok(ApiResponse { data: None, error: Some("An internal error occurred.".to_string())});
                    }
                }
            } else {
                error!(?error, "An unexpected error occurred during login.");
                response_option.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                return Ok(ApiResponse { data: None, error: Some("An internal error occurred.".to_string())});
            }
        }
    };

    let session_creation_result = create_session(user_id).await;
    if let Err(error) = session_creation_result {
        error!(?error);
        response_option.set_status(StatusCode::INTERNAL_SERVER_ERROR);
        return Ok(ApiResponse { data: None, error: Some("Failed to create user session.".to_string())});
    }

    let session_token = session_creation_result.ok().unwrap();
    let cookie_creation_result = set_session_cookie(&session_token);

    if let Err(error) = cookie_creation_result {
        error!(?error);
        response_option.set_status(StatusCode::INTERNAL_SERVER_ERROR);
        return Ok(ApiResponse { data: None, error: Some("Failed to set session cookie.".to_string())});
    }

    Ok(ApiResponse {
        data: Some("The user have been logged in successfully".to_string()),
        error: None,
    })
}
