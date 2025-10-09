use leptos::prelude::ServerFnError;
use leptos::*;

use crate::{auth::custom_auth::register_user, models::registration::FormData};

// TODO: Create an AppError enum and implement FromServerFnError for it
#[server(Register, "/auth")]
pub async fn register(form: FormData) -> Result<String, ServerFnError>{
    register_user(form).await
        .map_err(|e| ServerFnError::WrappedServerError(e))?;
    
    Ok("user have been successfully registered".to_string())
}
