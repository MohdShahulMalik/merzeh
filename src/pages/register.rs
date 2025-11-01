use garde::Validate;
use leptos::{html, prelude::*, reactive::spawn_local};

use crate::{models::{auth::RegistrationFormData, user::Identifier}, server_functions::auth::register};

#[component]
pub fn Register() -> impl IntoView {
    let (error, set_error) = signal("".to_string());
    let (success, set_success) = signal("".to_string());

    let name_input: NodeRef<html::Input> = NodeRef::new();
    let eamil_or_mobile_input: NodeRef<html::Input> = NodeRef::new();
    let password_input: NodeRef<html::Input> = NodeRef::new();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let name_value = name_input
            .get()
            .expect("<input> should be mounted")
            .value();
        let email_or_mobile_value = eamil_or_mobile_input
            .get()
            .expect("<input> should be mounted")
            .value();
        let password_value = password_input
            .get()
            .expect("<input> should be mounted")
            .value();

        let identifier = if email_or_mobile_value.contains('@') { Identifier::Email(email_or_mobile_value) } else { Identifier::Mobile(email_or_mobile_value) };

        let registration_form = RegistrationFormData {
            name: name_value,
            identifier,
            password: password_value,
        };

        match registration_form.validate(){
            Ok(_) => {},

            Err(report) => {
                set_error.set(format!("Validation Error: {}", report));
                return;
            }
            
        }

        spawn_local(async move{
            match register(registration_form).await {
                Ok(_) => set_success.set("Successful".to_string()),

                Err(e) => {
                    set_error.set(format!("Registration Error: {}", e));
                }
            }
        });

    };

    view! {
        <form on:submit = on_submit>

            <h1>"Create an Account"</h1>
            <p>"Sign up to get started"</p>

            <div class = "form-group">
                <label for = "name">"Full Name"</label>
                <input
                    type = "text"
                    name = "name"
                    placeholder = "Armaan Ali"
                    node_ref = name_input
                    required
                />
                <p>"Please enter your Name"</p>
            </div>

            <div class = "form-group">
                <label for = "contact">"Email or Mobile"</label>
                <input
                    type = "text"
                    name = "contact"
                    placeholder = "email@example.com or +91923XXXXX90"
                    node_ref = eamil_or_mobile_input
                    required
                />
                <p>"Enter a valid password or mobile number"</p>
            </div>

            <div class = "form-group">
                <label for = "password">"Password"</label>
                <input
                    type = "password"
                    name = "password"
                    node_ref = password_input
                    required
                />
                <p>"Password must contain 8 characters"</p>
            </div>

            <button
                class = "border-2 cursor-pointer"
                type = "submit">Create Account</button>

        </form>

        <Show
            when = move || !error.get().is_empty()
            fallback = view! {<p></p>}
        >
            <p>{error.get()}</p>
        </Show>

        <Show
            when = move || !success.get().is_empty()
            fallback = view! {<p></p>}
        >
            <p>{success.get()}</p>
        </Show>
        
    }
}
