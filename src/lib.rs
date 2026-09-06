#![recursion_limit = "256"]

#[cfg(feature = "ssr")]
use std::net::TcpListener;

#[cfg(feature = "ssr")]
use actix_files::Files;
#[cfg(feature = "ssr")]
use actix_web::dev::Server;
#[cfg(feature = "ssr")]
use actix_web::{App, HttpServer, web};
#[cfg(feature = "ssr")]
use leptos::config::{ConfFile, get_configuration};
#[cfg(feature = "ssr")]
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_actix::{LeptosRoutes, generate_route_list};
#[cfg(feature = "ssr")]
use leptos_meta::MetaTags;
#[cfg(feature = "ssr")]
use surrealdb::Surreal;
#[cfg(feature = "ssr")]
use surrealdb::engine::remote::ws::Client;

#[cfg(feature = "ssr")]
use crate::app::App;

pub mod app;
#[cfg(feature = "ssr")]
pub mod auth;
pub mod components;
#[cfg(feature = "ssr")]
pub mod database;
pub mod errors;
#[cfg(feature = "ssr")]
pub mod jobs;
pub mod models;
pub mod pages;
#[cfg(feature = "ssr")]
pub mod services;
#[cfg(feature = "ssr")]
pub mod utils;

pub mod server_functions;

#[cfg(feature = "ssr")]
fn run(addr: TcpListener, conf: ConfFile, db: Surreal<Client>) -> std::io::Result<Server> {
    let server = HttpServer::new(move || {
        // Generate the list of routes in your Leptos App
        let routes = generate_route_list(App);
        let leptos_options = &conf.leptos_options;
        let site_root = leptos_options.site_root.clone().to_string();

        App::new()
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", &site_root))
            // serve the favicon from /favicon.ico
            .service(favicon)
            .leptos_routes(routes, {
                let leptos_options = leptos_options.clone();
                move || {
                    view! {
                        <!DOCTYPE html>
                        <html lang="en">
                            <head>
                                <meta charset="utf-8"/>
                                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                                <AutoReload options=leptos_options.clone() />
                                <HydrationScripts options=leptos_options.clone()/>
                                <MetaTags/>
                            </head>
                            <body>
                                <App/>
                            </body>
                        </html>
                    }
                }
            })
            .app_data(web::Data::new(leptos_options.to_owned()))
            .app_data(web::Data::new(db.clone()))
    })
    .listen(addr)?
    .run();

    Ok(server)
}

#[cfg(feature = "ssr")]
#[actix_web::get("favicon.ico")]
async fn favicon(
    leptos_options: actix_web::web::Data<leptos::config::LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
    ))?)
}

#[cfg(feature = "ssr")]
pub fn spawn_app(db: Surreal<Client>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a available port");
    let port = listener
        .local_addr()
        .expect("Failed to get the port binded for the test")
        .port();
    let conf = get_configuration(Some("Cargo.toml")).unwrap();

    let server = run(listener, conf, db).expect("Failed to bind the address");
    let _handle = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
