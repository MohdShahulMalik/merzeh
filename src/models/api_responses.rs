use serde::{Deserialize, Serialize};

use crate::models::{
    mosque::PrayerTimes,
    user::{UserIdentifierOnClient, UserOnClient},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T = String> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn data(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            data: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct MosqueResponse {
    pub id: String,
    pub location: (f64, f64),
    pub name: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub cover_img: Option<String>,
    pub adhan_times: Option<PrayerTimes>,
    pub jamat_times: Option<PrayerTimes>,
    pub imam: Option<UserOnClient>,
    pub muazzin: Option<UserOnClient>,
    pub imam_contact: Vec<UserIdentifierOnClient>,
    pub muazzin_contact: Vec<UserIdentifierOnClient>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum MixedMosqueResponse {
    MosquesVec(Vec<MosqueResponse>),
    SingleMosque(MosqueResponse),
}
