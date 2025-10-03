use anyhow::{anyhow, Context, Result};
use garde::Validate;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tokio::runtime::Runtime;

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct FormData {
    #[garde(dive)]
    pub identifier: UserIdentifier,
    #[garde(length(min = 8))]
    pub password: String
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub enum UserIdentifier {
    Email(
        #[garde(email)]
        String
    ),
    Mobile(
        #[garde(pattern(r"^[6-9][0-9]{9}$"))]
        String
    ),
}

impl FormData {
    pub async fn validate_uniqueness(&self, db: &&'static Surreal<Client>) -> Result<()> {
        let (field, value) = match &self.identifier {
            UserIdentifier::Email(email) => ("email", email.to_string()),
            UserIdentifier::Mobile(mobile) => ("mobile", mobile.to_string()),
        };

        let query_str = format!("SELECT * FROM user WHERE {} = $value", field);
        let mut result = db
            .query(&query_str)
            .bind(("value", value))
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?;

        let res: Vec<serde_json::Value> = result
            .take(0)
            .map_err(|_| anyhow!("Failed to parse query result"))?;

        if !res.is_empty() {
            Err(anyhow!("{} already exists", field))
        } else {
            Ok(())
        }
    }
}
