use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use surrealdb::sql::Geometry;

#[derive(Debug, Serialize, Deserialize)]
pub struct Mosque {
    pub id: RecordId,
    pub name: String,
    pub location: Geometry,
    pub street: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MosquesResponse {
    pub elements: Vec<MosqueElement>,
}

#[derive(Debug, Deserialize)]
pub struct MosqueElement {
    #[serde(rename = "type")]
    pub element_type: String,
    pub id: i64,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub center: Option<Center>,
    pub tags: Option<Tags>,
}

#[derive(Debug, Deserialize)]
pub struct Center{
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Deserialize)]
pub struct Tags{
    pub name: Option<String>,
    #[serde(rename = "addr:street")]
    pub street: Option<String>,
    #[serde(rename = "addr:city")]
    pub city: Option<String>,
}
