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
use crate::auth::session::{create_session, set_session_cookie};
#[cfg(feature = "ssr")]
use crate::auth::custom_auth::register_user;
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

/* pub async fn login(
    form: LoginFormData,
) {
    let db = get_db();

    let query = r#"
        
    "#;
} */
