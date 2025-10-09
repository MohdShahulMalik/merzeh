use garde::Validate;
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub display_name: String,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: RecordId,
    pub created_at: Datetime,
    pub display_name: String,
    pub password_hash: String,
    pub role: String,
    pub updated_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub updated_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserIdentifier {
    #[serde(flatten)]
    pub identifier: Identifier,
    pub user_id: RecordId,
}

#[derive(Debug, Validate, Deserialize, Serialize, Clone)]
#[serde(tag = "identifier_type", content = "identifier_value")]
pub enum Identifier {
    #[serde(rename = "email")]
    Email(#[garde(email)] String),
    #[serde(rename = "mobile")]
    Mobile(#[garde(pattern(r"^[6-9][0-9]{9}$"))] String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserIdentifier {
    #[serde(flatten)]
    pub identifier: Identifier,
    pub user_id: RecordId,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}
