use crate::server_functions::mosque::add_mosques_of_region;
use leptos::{html, prelude::*, reactive::spawn_local};

#[component]
pub fn AddMosquesOfRegion() -> impl IntoView {
    let (error, set_error) = signal("".to_string());
    let (success, set_success) = signal("".to_string());

    let south_input: NodeRef<html::Input> = NodeRef::new();
    let west_input: NodeRef<html::Input> = NodeRef::new();
    let north_input: NodeRef<html::Input> = NodeRef::new();
    let east_input: NodeRef<html::Input> = NodeRef::new();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let south_value = south_input.get().expect("<input> should be mounted").value();
        let west_value = west_input.get().expect("<input> should be mounted").value();
        let north_value = north_input.get().expect("<input> should be mounted").value();
        let east_value = east_input.get().expect("<input> should be mounted").value();

        let south = match south_value.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                set_error.set("Invalid 'South' coordinate. Must be a number.".to_string());
                return;
            }
        };
        let west = match west_value.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                set_error.set("Invalid 'West' coordinate. Must be a number.".to_string());
                return;
            }
        };
        let north = match north_value.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                set_error.set("Invalid 'North' coordinate. Must be a number.".to_string());
                return;
            }
        };
        let east = match east_value.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                set_error.set("Invalid 'East' coordinate. Must be a number.".to_string());
                return;
            }
        };
        
        set_error.set("".to_string());

        spawn_local(async move {
            match add_mosques_of_region(south, west, north, east).await {
                Ok(response) => {
                    if let Some(err_msg) = response.error {
                        set_error.set(format!("Server Error: {}", err_msg));
                        set_success.set("".to_string());
                    } else if let Some(data_msg) = response.data {
                        set_success.set(format!("Success: {}", data_msg));
                        set_error.set("".to_string());
                    } else {
                        set_error.set("Received an empty response from server.".to_string());
                        set_success.set("".to_string());
                    }
                },
                Err(e) => {
                    set_error.set(format!("Function Error: {}", e));
                    set_success.set("".to_string());
                }
            }
        });
    };

    view! {
        <main>
            <form on:submit=on_submit>
                <h1>"Add Mosques in a Region"</h1>
                <p>"Enter the bounding box coordinates to fetch and store mosques from OpenStreetMap."</p>

                <div class="form-group">
                    <label for="south">"South"</label>
                    <input
                        type="text"
                        name="south"
                        placeholder="-90.0"
                        node_ref=south_input
                        required
                    />
                </div>
                <div class="form-group">
                    <label for="west">"West"</label>
                    <input
                        type="text"
                        name="west"
                        placeholder="-180.0"
                        node_ref=west_input
                        required
                    />
                </div>
                <div class="form-group">
                    <label for="north">"North"</label>
                    <input
                        type="text"
                        name="north"
                        placeholder="90.0"
                        node_ref=north_input
                        required
                    />
                </div>
                <div class="form-group">
                    <label for="east">"East"</label>
                    <input
                        type="text"
                        name="east"
                        placeholder="180.0"
                        node_ref=east_input
                        required
                    />
                </div>

                <button type="submit">"Fetch and Add Mosques"</button>
            </form>

            <Show
                when=move || !error.get().is_empty()
                fallback=|| view! { <p></p> }
            >
                <p style="color: red;">{error.get()}</p>
            </Show>

            <Show
                when=move || !success.get().is_empty()
                fallback=|| view! { <p></p> }
            >
                <p style="color: green;">{success.get()}</p>
            </Show>
        </main>
    }
}
