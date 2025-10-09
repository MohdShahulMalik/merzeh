use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use surrealdb::sql::Datetime;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSession {
    pub user_id: RecordId,
    pub session_token: String,
    pub expires_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: RecordId,
    pub user_id: RecordId,
    pub session_token: String,
    pub expires_at: Datetime,
    pub created_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSession {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<Datetime>,
}
