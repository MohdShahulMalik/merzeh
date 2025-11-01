use crate::models::user::Identifier;
use garde::Validate;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::errors::registration::RegistrationError;
#[cfg(feature = "ssr")]
use anyhow::{Result, anyhow};
#[cfg(feature = "ssr")]
use crate::database::connection::get_db;

#[derive(Debug, Validate, Deserialize, Serialize, Clone)]
pub struct RegistrationFormData {
    #[garde(length(min = 2, max = 100))]
    pub name: String,
    #[garde(dive)]
    pub identifier: Identifier,
    #[garde(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Validate, Deserialize, Serialize, Clone)]
pub struct LoginFormData {
    #[garde(dive)]
    pub identifier: Identifier,
    #[garde(length(min = 8))]
    pub password: String,
}

#[cfg(feature = "ssr")]
impl RegistrationFormData {
    pub async fn validate_uniqueness(&self) -> Result<()> {
        let db = get_db();
        
        let (field, value) = match &self.identifier {
            Identifier::Email(email) => ("email", email.to_string()),
            Identifier::Mobile(mobile) => ("mobile", mobile.to_string()),
        };

        let query_str = format!("SELECT * FROM user WHERE {} = $value", field);
        let mut result = db
            .query(&query_str)
            .bind(("value", value))
            .await
            .map_err(|e| RegistrationError::DatabaseError(Box::new(e)))?;

        let res: Vec<serde_json::Value> = result
            .take(0)
            .map_err(|_| anyhow!("Failed to parse query result"))?;

        if !res.is_empty() {
            Err(RegistrationError::NotUniqueError(field.to_string()))?
        } else {
            Ok(())
        }
    }
}
