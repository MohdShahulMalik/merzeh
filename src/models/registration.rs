use anyhow::{anyhow, Result};
use garde::Validate;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use crate::models::user::Identifier;

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct FormData {
    #[garde(length(min = 2, max = 100))]
    pub name: String,
    #[garde(dive)]
    pub identifier: Identifier,
    #[garde(length(min = 8))]
    pub password: String
}

impl FormData {
    pub async fn validate_uniqueness(&self, db: &&'static Surreal<Client>) -> Result<()> {
        let (field, value) = match &self.identifier {
            Identifier::Email(email) => ("email", email.to_string()),
            Identifier::Mobile(mobile) => ("mobile", mobile.to_string()),
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
