use std::env;
use anyhow::Context;
use once_cell::sync::OnceCell;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::runtime::Runtime;
use dotenvy::dotenv;

static DB: OnceCell<Surreal<Client>> = OnceCell::new();

pub fn get_db() -> &'static Surreal<Client> {
    DB.get_or_init(|| {
        dotenv().ok();

        let rt = Runtime::new().expect("Failed to create a new tokio runtime");
        rt.block_on(async {
            let db_url = env::var("SURREAL_URL").with_context(|| "SURREAL_URL must be set").unwrap();
            let db_user = env::var("SURREAL_USER").with_context(|| "SURREAL_USER must be set").unwrap();
            let db_pass = env::var("SURREAL_PASS").with_context(|| "SURREAL_PASS must be set").unwrap();
            let db_name = env::var("SURREAL_DB").with_context(|| "SURREAL_DB must be set").unwrap();
            let db_ns = env::var("SURREAL_NS").with_context(|| "SURREAL_NS must be set").unwrap();
                        
            let db = Surreal::new::<Ws>(&db_url)
                .await
                .expect("Failed to connect to database");

            db.signin(Root {
                username: &db_user,
                password: &db_pass,
            })
            .await
            .expect("Failed to sign in to database");

            db.use_ns(&db_ns)
                .use_db(&db_name)
                .await
                .expect("Failed to use namespace or database");

            db
        })
    })
} 

