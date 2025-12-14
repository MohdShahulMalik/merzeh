use leptos::*;
use crate::models::form::InputConfig;
use leptos::prelude::*;
use leptos::{ev::Event};

#[component]
pub fn InputField(form_config: InputConfig, on_change: Option<impl Fn(String) + 'static>) -> impl IntoView {
    match form_config.input_type.as_str() {
        "text" => {
            view! {
                <div class = "field-group">
                    <label>{form_config.label}</label>
                    <input
                        type = "text"
                        name = {form_config.name.clone()}
                        prop:value = {form_config.default_value.clone().unwrap_or_default()}
                        placeholder = {form_config.placeholder.clone().unwrap_or_default()}
                        on:change = move |ev: Event| {
                            let value = event_target_value(&ev);
                            if let Some(on_change) = &on_change {
                                on_change(value);
                            }
                        }
                    />
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
