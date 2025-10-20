use crate::server_functions::auth::register;
use crate::models::auth::RegistrationFormData;
use leptos::*;

// Or at minimum, ensure it's exported and used:
#[component]
pub fn RegisterForm() -> impl IntoView {
    let register_action = create_action(|form: &RegistrationFormData| {
        let form = form.clone();
        async move {
            register(form).await  // This reference ensures the fn is compiled
        }
    });
    // ...
}
