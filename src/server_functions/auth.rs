#[cfg(feature = "ssr")]
use crate::auth::oauth::discord::DiscordProvider;
#[cfg(feature = "ssr")]
use crate::auth::oauth::helpers::OAuthCallback;
#[cfg(feature = "ssr")]
use crate::auth::oauth::microsoft::MicrosoftProvider;
use crate::models::auth::LoginFormData;
#[cfg(feature = "ssr")]
use crate::models::auth::Platform;
#[cfg(feature = "ssr")]
use crate::models::oauth::GoogleUser;
use crate::models::{api_responses::ApiResponse, auth::RegistrationFormData, user::UserOnClient};
#[cfg(feature = "ssr")]
use garde::Validate;
use leptos::prelude::ServerFnError;
use leptos::server_fn::codec::{DeleteUrl, Json};
use leptos::*;

#[cfg(feature = "ssr")]
use crate::auth::custom_auth::{authenticate, register_user};
#[cfg(feature = "ssr")]
use crate::auth::oauth::google::{
    exchange_code, find_or_create_user, get_authorization_url, get_user_info,
};
#[cfg(feature = "ssr")]
use crate::auth::oauth::state::{generate_state, validate_state};
#[cfg(feature = "ssr")]
use crate::auth::session::{
    create_session, delete_session, remove_session_cookie, set_session_cookie,
};
#[cfg(feature = "ssr")]
use crate::errors::auth::AuthError;
#[cfg(feature = "ssr")]
use crate::errors::session::SessionError;
#[cfg(feature = "ssr")]
use crate::utils::ssr::{ServerResponse, get_authenticated_user, get_server_context};
#[cfg(feature = "ssr")]
use actix_web::HttpRequest;
#[cfg(feature = "ssr")]
use tracing::error;

#[server(input = Json, output = Json, prefix = "/auth", endpoint = "register")]
pub async fn register(form: RegistrationFormData) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db) = match get_server_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let validation_result = form.validate();

    if let Err(error) = validation_result {
        let errors = error
            .iter()
            .map(|(field, msg)| format!("{}, {}", field, msg))
            .collect::<Vec<_>>();
        error!(?errors);
        return Ok(responder.unprocessable_entity(errors.join("\n")));
    }

    let validation_result_for_uniqueness = form.validate_uniqueness(&db).await;
    if let Err(error) = validation_result_for_uniqueness {
        error!(?error);
        return Ok(responder.conflict(format!("{}", error)));
    }

    let registration_result = register_user(form.clone(), &db).await;

    if let Err(error) = registration_result {
        error!(?error, "Failed to register the user");
        return Err(ServerFnError::ServerError(
            "Failed to register the user".to_string(),
        ));
    };

    let user_id = registration_result.ok();
    let session_creation_result = create_session(user_id.unwrap(), &db).await;
    if let Err(error) = session_creation_result {
        error!(?error);
        return Err(ServerFnError::ServerError(
            "Failed to generate session tokens for the registered user".to_string(),
        ));
    }

    let session_token = session_creation_result.ok().unwrap();

    if let Platform::Web = form.platform {
        let cookie_creation_result = set_session_cookie(&session_token);

        if let Err(error) = cookie_creation_result {
            error!(?error);
            return Err(ServerFnError::ServerError(
                "Failed to create appropriate cookies after registration".to_string(),
            ));
        }

        Ok(responder.ok("The user has been registered successfully".to_string()))
    } else {
        Ok(responder.ok(session_token))
    }
}

#[server(input = Json, output = Json, prefix = "/auth", endpoint = "login")]
pub async fn login(form: LoginFormData) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, _user) = match get_authenticated_user::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let user_id = match authenticate(form.clone(), &db).await {
        Ok(id) => id,
        Err(error) => {
            if let Some(auth_error) = error.downcast_ref::<AuthError>() {
                match auth_error {
                    AuthError::UserNotFound | AuthError::PasswordVerificationError(_) => {
                        error!("Authentication failed for user.");
                        return Ok(
                            responder.unauthorized("Invalid username or password.".to_string())
                        );
                    }
                    AuthError::DatabaseError(_) | AuthError::PasswordHashError(_) => {
                        error!(?error, "Internal server error during authentication.");
                        return Ok(responder
                            .internal_server_error("An internal error occurred.".to_string()));
                    }
                    _ => {
                        error!(?error, "An unexpected authentication error occurred.");
                        return Ok(responder
                            .internal_server_error("An internal error occurred.".to_string()));
                    }
                }
            } else {
                error!(?error, "An unexpected error occurred during login.");
                return Ok(
                    responder.internal_server_error("An internal error occurred.".to_string())
                );
            }
        }
    };

    let session_creation_result = create_session(user_id, &db).await;
    if let Err(error) = session_creation_result {
        error!(?error);
        return Ok(responder.internal_server_error("Failed to create user session.".to_string()));
    }

    let session_token = session_creation_result.ok().unwrap();

    if let Platform::Web = form.platform {
        let cookie_creation_result = set_session_cookie(&session_token);

        if let Err(error) = cookie_creation_result {
            error!(?error);
            return Ok(responder.internal_server_error("Failed to set session cookie.".to_string()));
        }

        Ok(responder.ok("The user has been logged in successfully".to_string()))
    } else {
        Ok(responder.ok(session_token))
    }
}

#[server(input = Json, output = Json, prefix = "/auth", endpoint = "me")]
pub async fn fetch_me() -> Result<ApiResponse<UserOnClient>, ServerFnError> {
    let (response_options, _db, user) = match get_authenticated_user::<UserOnClient>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    Ok(responder.ok(UserOnClient::from(user)))
}

#[server(input=DeleteUrl, output=Json, prefix="/auth", endpoint="logout")]
pub async fn logout() -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, _user) = match get_authenticated_user::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let req = match leptos_actix::extract::<HttpRequest>().await {
        Ok(req) => req,
        Err(e) => {
            error!(?e, "Failed to extract request");
            return Ok(responder.internal_server_error("Internal server error".to_string()));
        }
    };

    let session_token = if let Some(cookie) = req.cookie("__Host-session") {
        cookie.value().to_string()
    } else if let Some(auth_header) = req.headers().get("Authorization") {
        let auth_str = auth_header.to_str().unwrap_or("");
        if auth_str.starts_with("Bearer ") {
            auth_str.trim_start_matches("Bearer ").to_string()
        } else {
            return Ok(responder.unauthorized("You are not logged in".to_string()));
        }
    } else {
        return Ok(responder.unauthorized("You are not logged in".to_string()));
    };

    if let Err(e) = delete_session(&session_token, &db).await {
        error!(?e, "Failed to delete session");
        let session_error_option = e.downcast_ref::<SessionError>();
        if let Some(session_error) = session_error_option {
            match session_error {
                SessionError::SessionExpired(_) => {
                    return Ok(responder.unauthorized("Your session has expired".to_string()));
                }
                SessionError::SessionNotFound => {
                    return Ok(responder.unauthorized("Session not found".to_string()));
                }
                SessionError::InvalidToken => {
                    return Ok(responder.unauthorized("Invalid session token".to_string()));
                }
                SessionError::UserNotFound => {
                    return Ok(
                        responder.unauthorized("User not found for this session".to_string())
                    );
                }
                SessionError::DatabaseError(err) => {
                    return Ok(responder
                        .internal_server_error(format!("Database error occurred: {}", err)));
                }
            }
        }
        return Ok(responder
            .internal_server_error("Failed to delete the session from the server".to_string()));
    }

    // Only attempt to remove cookie if it was present
    if req.cookie("__Host-session").is_some() {
        if let Err(e) = remove_session_cookie() {
            error!(?e, "Failed to remove session cookie");
            return Ok(
                responder.internal_server_error("Failed to remove session cookie".to_string())
            );
        }
    }

    Ok(responder.ok("Successfully logged out the user".to_string()))
}

#[server(input = Json, output = Json, prefix = "/auth", endpoint = "google-url")]
pub async fn get_google_oauth_url() -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, _db) = match get_server_context().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let state = match generate_state() {
        Ok(s) => s,
        Err(e) => {
            error!(?e, "Failed to generate state");
            return Ok(responder
                .internal_server_error("Failed to generate authentication state".to_string()));
        }
    };

    let url = match get_authorization_url(&state) {
        Ok(u) => u,
        Err(e) => {
            error!(?e, "Failed to get authorization URL");
            return Ok(
                responder.internal_server_error("Failed to create authorization URL".to_string())
            );
        }
    };

    let cookie = format!(
        "google_oauth_state={}; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age={}",
        state,
        10 * 60
    );

    use actix_web::http::header::{HeaderValue, SET_COOKIE};

    let header_value = match HeaderValue::from_str(&cookie) {
        Ok(v) => v,
        Err(e) => {
            error!(?e, "Failed to create header value");
            return Ok(responder.internal_server_error("Failed to set cookie".to_string()));
        }
    };

    responder.insert_header(SET_COOKIE, header_value);

    Ok(responder.ok(url))
}

#[server(input = Json, output = Json, prefix = "/auth", endpoint = "google-callback")]
pub async fn handle_google_callback(
    code: String,
    state: String,
) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db) = match get_server_context().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let req = match leptos_actix::extract::<HttpRequest>().await {
        Ok(req) => req,
        Err(e) => {
            error!(?e, "Failed to extract request");
            return Ok(responder.internal_server_error("Internal server error".to_string()));
        }
    };

    let stored_state = req
        .cookie("google_oauth_state")
        .map(|c| c.value().to_string())
        .unwrap_or_default();

    if !validate_state(&state, &stored_state) {
        error!("State validation failed");
        return Ok(responder.bad_request("Invalid authentication state".to_string()));
    }

    let token_response = match exchange_code(&code).await {
        Ok(token) => token,
        Err(e) => {
            error!(?e, "Failed to exchange code");
            return Ok(responder.bad_request("Failed to exchange authorization code".to_string()));
        }
    };

    let user_info: GoogleUser = match get_user_info(&token_response.access_token).await {
        Ok(user) => user,
        Err(e) => {
            error!(?e, "Failed to get user info");
            return Ok(responder.bad_request("Failed to get user information".to_string()));
        }
    };

    let user_id = match find_or_create_user(user_info, &db).await {
        Ok(id) => id,
        Err(e) => {
            error!(error = %e, "Failed to find or create user");
            return Err(ServerFnError::ServerError(format!(
                "Failed to authenticate user: {:?}",
                e
            )));
        }
    };

    let session_token = match create_session(user_id, &db).await {
        Ok(token) => token,
        Err(e) => {
            error!(?e, "Failed to create session");
            return Err(ServerFnError::ServerError(
                "Failed to create session".to_string(),
            ));
        }
    };

    use actix_web::http::header::{HeaderValue, SET_COOKIE};

    let session_cookie = format!(
        "__Host-session={}; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age={}",
        session_token,
        24 * 60 * 60
    );

    let clear_state_cookie =
        "google_oauth_state=; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age=0";

    if let Ok(session_header) = HeaderValue::from_str(&session_cookie) {
        responder.append_header(SET_COOKIE, session_header);
    }

    if let Ok(clear_header) = HeaderValue::from_str(clear_state_cookie) {
        responder.append_header(SET_COOKIE, clear_header);
    }

    Ok(responder.ok("Successfully authenticated with Google".to_string()))
}

#[server(input = Json, output = Json, prefix = "/auth", endpoint = "discord-url")]
pub async fn get_discord_oauth_url() -> Result<ApiResponse<String>, ServerFnError> {
    OAuthCallback::get_url::<DiscordProvider>("discord_oauth_state").await
}

#[server(input = Json, output = Json, prefix = "/auth", endpoint = "discord-callback")]
pub async fn handle_discord_callback(
    code: String,
    state: String,
) -> Result<ApiResponse<String>, ServerFnError> {
    OAuthCallback::handle::<DiscordProvider>(code, state, "discord_oauth_state").await
}

#[server(input = Json, output = Json, prefix = "/auth", endpoint = "microsoft-url")]
pub async fn get_microsoft_oauth_url() -> Result<ApiResponse<String>, ServerFnError> {
    OAuthCallback::get_url::<MicrosoftProvider>("microsoft_oauth_state").await
}

#[server(input = Json, output = Json, prefix = "/auth", endpoint = "microsoft-callback")]
pub async fn handle_microsoft_callback(
    code: String,
    state: String,
) -> Result<ApiResponse<String>, ServerFnError> {
    OAuthCallback::handle::<MicrosoftProvider>(code, state, "microsoft_oauth_state").await
}
