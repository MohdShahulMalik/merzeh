use garde::Validate;
use leptos::{html, prelude::*, reactive::spawn_local};
use leptos_router::components::A;

use crate::models::{auth::{LoginFormData, RegistrationFormData}, user::Identifier};
use crate::server_functions::auth::{login, register};

#[component]
pub fn Register() -> impl IntoView {
    let (error, set_error) = signal("".to_string());
    let (success, set_success) = signal("".to_string());
    let (name_error, set_name_error) = signal(String::new());
    let (password_error, set_password_error) = signal(String::new());
    let (identifier_error, set_identifier_error) = signal(String::new());

    let name_input: NodeRef<html::Input> = NodeRef::new();
    let email_or_mobile_input: NodeRef<html::Input> = NodeRef::new();
    let password_input: NodeRef<html::Input> = NodeRef::new();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        // Clear previous errors
        set_name_error.set(String::new());
        set_identifier_error.set(String::new());
        set_password_error.set(String::new());
        set_error.set(String::new());

        let name_value = name_input.get().expect("<input> should be mounted").value();
        let email_or_mobile_value = email_or_mobile_input
            .get()
            .expect("<input> should be mounted")
            .value();
        let password_value = password_input
            .get()
            .expect("<input> should be mounted")
            .value();

        let identifier = if email_or_mobile_value.contains('@') {
            Identifier::Email(email_or_mobile_value)
        } else {
            Identifier::Mobile(email_or_mobile_value)
        };

        let registration_form = RegistrationFormData {
            name: name_value,
            identifier,
            password: password_value,
        };

        if let Err(report) = registration_form.validate() {
            for (field, error) in report.iter() {
                let field_str = field.to_string();
                let error_msg = error.to_string();

                if field_str.starts_with("name") {
                    set_name_error.set(error_msg);
                } else if field_str.starts_with("identifier") {
                    set_identifier_error.set(error_msg);
                } else if field_str.starts_with("password") {
                    set_password_error.set(error_msg);
                }
            }
            return;
        }

        spawn_local(async move {
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

            <h1>Create an Account</h1>
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
                <Show when = move || !name_error.get().is_empty()>
                    <p>{name_error.get()}</p>
                </Show>
            </div>

            <div class = "form-group">
                <label for = "contact">"Email or Mobile"</label>
                <input
                    type = "text"
                    name = "contact"
                    placeholder = "email@example.com or +91923XXXXX90"
                    node_ref = email_or_mobile_input
                    required
                />
                <Show when = move || !identifier_error.get().is_empty()>
                    <p>{identifier_error.get()}</p>
                </Show>
                <Show when = move || identifier_error.get().is_empty()>
                    <p>"Enter a valid email or mobile number"</p>
                </Show>
            </div>

            <div class = "form-group">
                <label for = "password">"Password"</label>
                <input
                    type = "password"
                    name = "password"
                    node_ref = password_input
                    required
                />
                <Show when = move || !password_error.get().is_empty()>
                    <p>{password_error.get()}</p>
                </Show>
                <Show when = move || password_error.get().is_empty()>
                    <p>"Password must contain 8 characters"</p>
                </Show>
            </div>

            <button
                class = "border-2 cursor-pointer bg-primary"
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

#[component]
pub fn Login() -> impl IntoView {
    let (error, set_error) = signal("".to_string());
    let (success, set_success) = signal("".to_string());
    let (identifier_error, set_identifier_error) = signal(String::new());
    let (password_error, set_password_error) = signal(String::new());

    let email_or_mobile_input: NodeRef<html::Input> = NodeRef::new();
    let password_input: NodeRef<html::Input> = NodeRef::new();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        // Clear previous errors
        set_identifier_error.set(String::new());
        set_password_error.set(String::new());
        set_error.set(String::new());

        let email_or_mobile_value = email_or_mobile_input
            .get()
            .expect("failed to get the the email or mobile input node unfortunately")
            .value();

        let password_value = password_input
            .get()
            .expect("Failed to get the password input node unfortunately")
            .value();

        let identifier = if email_or_mobile_value.contains('@') {
            Identifier::Email(email_or_mobile_value)
        } else {
            Identifier::Mobile(email_or_mobile_value)
        };
        let login_form = LoginFormData {
            identifier,
            password: password_value,
        };

        if let Err(report) = login_form.validate() {
            for (field, error) in report.iter() {
                let field_str = field.to_string();
                let error_msg = error.to_string();

                if field_str.starts_with("identifier") {
                    set_identifier_error.set(error_msg);
                } else if field_str.starts_with("password") {
                    set_password_error.set(error_msg);
                }
            }
            return;
        }

        spawn_local(async move {
            match login(login_form).await {
                Ok(_) => set_success.set("Successful".to_string()),

                Err(e) => set_error.set(format!("Error: {}", e)),
            }
        });
    };

    view! {
        <main class = "flex gap-1 h-svh bg-surface-900">
            <section class = "felx-[2] content-center grid gap-16 pl-24">

                <div class = "flex gap-2">
                    <img class = "w-auto h-16 rounded-full" src = "/assets/logo.png" />

                    <div class = "w-full">
                        <picture>
                          <source srcset="/assets/logo-text-dark.png" media="(prefers-color-scheme: dark)" />
                          <source srcset="/assets/logo-text-light.png" media="(prefers-color-scheme: light)" />
                          <img class = "w-auto h-12" src="/assets/logo-text-light.png" alt="Merzah logo" />
                        </picture>
                        <span class = "text-foreground-600">Community Management App</span>
                    </div>

                </div>

                <div class = "w-[60%] grid gap-16">
                    <p class = "text-4xl font-bold text-foreground-900">
                        "Welcome back to a simpler way to manage your community"
                    </p>

                    <p class = "text-foreground-600">
                        "A modern, trustworthy experience that respects Islamic design. warm tones, gentle patterns, and clear usability, all in one place."
                    </p>
                </div>

            </section>

            <section class = "flex-1 bg-surface-700 h-[85svh]">
                <form on:submit = on_submit>
                    <h1>"Login"</h1>
                    <h2>"Welcome back. please enter your details."</h2>

                    <div class = "form-group">
                        <label for = "contact">"Email or Mobile"</label>
                        <input
                            type = "text"
                            name = "contact"
                            placeholder = "email@example.com or +91923XXXXX90"
                            node_ref = email_or_mobile_input
                            required
                        />
                        <Show when = move || !identifier_error.get().is_empty()>
                            <p>{identifier_error.get()}</p>
                        </Show>
                    </div>

                    <div class = "form-group">
                        <label for = "password">"Password"</label>
                        <input
                            type = "password"
                            name = "password"
                            node_ref = password_input
                            required
                        />
                        <Show when = move || !password_error.get().is_empty()>
                            <p>{password_error.get()}</p>
                        </Show>
                        <Show when = move || password_error.get().is_empty()>
                            <p>"Password must contain 8 characters"</p>
                        </Show>
                            <div>
                                <A href = "/forgot-password">"Forgot Password"</A>
                            </div>
                    </div>


                    <button
                        class = "border-2 bg-primary border-stroke cursor-pointer"
                        type = "submit">Login
                    </button>


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


                <p>"Don't have an account?"</p>
                <A href = "/register"><button>Register</button></A>
            </section>

        </main>
    }
}
