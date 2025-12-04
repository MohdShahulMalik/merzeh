use leptos::*;
use crate::models::form::FormConfig;
use leptos::prelude::*;

#[component]
pub fn FormField(form_config: FormConfig) -> impl IntoView {
    match form_config.input_type.as_str() {
        "text" => {
            view! {
                <div class = "field-group">
                    <label>{form_config.name}</label>
                    <input type = "text" />
                </div>
            }.into_any()
        }

        _ => {
            view! {
                <label>Not a valid field</label>
            }.into_any()
        }
    }
}
