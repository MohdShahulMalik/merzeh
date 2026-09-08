#[cfg(feature = "ssr")]
use crate::auth::session::get_user_by_session;
use crate::models::api_responses::ApiResponse;
#[cfg(feature = "ssr")]
use crate::models::user::User;
#[cfg(feature = "ssr")]
use actix_web::{http::StatusCode, web};
#[cfg(feature = "ssr")]
use leptos::prelude::use_context;
#[cfg(feature = "ssr")]
use leptos_actix::ResponseOptions;
#[cfg(feature = "ssr")]
use surrealdb::{Surreal, engine::remote::ws::Client};
#[cfg(feature = "ssr")]
use tracing::error;

#[cfg(feature = "ssr")]
pub async fn get_server_context<T>() -> Result<(ResponseOptions, Surreal<Client>), ApiResponse<T>> {
    let response_options = match use_context::<ResponseOptions>() {
        Some(ro) => ro,
        None => {
            error!("Failed to get ResponseOptions from context");
            return Err(ApiResponse::error("Internal Server Error".to_string()));
        }
    };

    let db = match leptos_actix::extract::<web::Data<Surreal<Client>>>().await {
        Ok(db) => db,
        Err(e) => {
            error!(?e, "Failed to extract database client");
            response_options.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ApiResponse::error("Internal Server Error".to_string()));
        }
    };

    Ok((response_options, db.get_ref().clone()))
}

#[cfg(feature = "ssr")]
pub async fn get_authenticated_user_and_context<T>()
-> Result<(ResponseOptions, Surreal<Client>, User), ApiResponse<T>> {
    let (response_options, db) = get_server_context::<T>().await?;

    let req = match leptos_actix::extract::<actix_web::HttpRequest>().await {
        Ok(req) => req,
        Err(e) => {
            error!(?e, "Failed to extract request");
            response_options.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ApiResponse::error("Internal Server Error".to_string()));
        }
    };

    let session_token = if let Some(cookie) = req.cookie("__Host-session") {
        cookie.value().to_string()
    } else if let Some(auth_header) = req.headers().get("Authorization") {
        let auth_str = auth_header.to_str().unwrap_or("");
        if auth_str.starts_with("Bearer ") {
            auth_str.trim_start_matches("Bearer ").to_string()
        } else {
            response_options.set_status(StatusCode::UNAUTHORIZED);
            return Err(ApiResponse::error("You are not logged in".to_string()));
        }
    } else {
        response_options.set_status(StatusCode::UNAUTHORIZED);
        return Err(ApiResponse::error("You are not logged in".to_string()));
    };

    let user = match get_user_by_session(&session_token, &db).await {
        Ok(user) => user,
        Err(e) => {
            error!(?e, "Failed to get user by session");
            response_options.set_status(StatusCode::UNAUTHORIZED);
            return Err(ApiResponse::error("Invalid or expired session".to_string()));
        }
    };

    Ok((response_options, db, user))
}

#[cfg(feature = "ssr")]
pub struct ServerResponse {
    options: ResponseOptions,
}

#[cfg(feature = "ssr")]
impl ServerResponse {
    pub fn new(options: ResponseOptions) -> Self {
        Self { options }
    }

    pub fn insert_header(
        &self,
        name: actix_web::http::header::HeaderName,
        value: actix_web::http::header::HeaderValue,
    ) {
        self.options.insert_header(name, value);
    }

    pub fn append_header(
        &self,
        name: actix_web::http::header::HeaderName,
        value: actix_web::http::header::HeaderValue,
    ) {
        self.options.append_header(name, value);
    }

    pub fn ok<T>(&self, data: T) -> ApiResponse<T> {
        self.options.set_status(StatusCode::OK);
        ApiResponse::data(data)
    }

    pub fn created<T>(&self, data: T) -> ApiResponse<T> {
        self.options.set_status(StatusCode::CREATED);
        ApiResponse::data(data)
    }

    pub fn accepted<T>(&self, data: T) -> ApiResponse<T> {
        self.options.set_status(StatusCode::ACCEPTED);
        ApiResponse::data(data)
    }

    pub fn no_content<T>(&self, data: T) -> ApiResponse<T> {
        self.options.set_status(StatusCode::NO_CONTENT);
        ApiResponse::data(data)
    }

    pub fn bad_request<T>(&self, error: String) -> ApiResponse<T> {
        self.options.set_status(StatusCode::BAD_REQUEST);
        ApiResponse::error(error)
    }

    pub fn unauthorized<T>(&self, error: String) -> ApiResponse<T> {
        self.options.set_status(StatusCode::UNAUTHORIZED);
        ApiResponse::error(error)
    }

    pub fn forbidden<T>(&self, error: String) -> ApiResponse<T> {
        self.options.set_status(StatusCode::FORBIDDEN);
        ApiResponse::error(error)
    }

    pub fn not_found<T>(&self, error: String) -> ApiResponse<T> {
        self.options.set_status(StatusCode::NOT_FOUND);
        ApiResponse::error(error)
    }

    pub fn method_not_allowed<T>(&self, error: String) -> ApiResponse<T> {
        self.options.set_status(StatusCode::METHOD_NOT_ALLOWED);
        ApiResponse::error(error)
    }

    pub fn unprocessable_entity<T>(&self, error: String) -> ApiResponse<T> {
        self.options.set_status(StatusCode::UNPROCESSABLE_ENTITY);
        ApiResponse::error(error)
    }

    pub fn internal_server_error<T>(&self, error: String) -> ApiResponse<T> {
        self.options.set_status(StatusCode::INTERNAL_SERVER_ERROR);
        ApiResponse::error(error)
    }

    pub fn conflict<T>(&self, error: String) -> ApiResponse<T> {
        self.options.set_status(StatusCode::CONFLICT);
        ApiResponse::error(error)
    }

    pub fn service_unavailable<T>(&self, error: String) -> ApiResponse<T> {
        self.options.set_status(StatusCode::SERVICE_UNAVAILABLE);
        ApiResponse::error(error)
    }
}
