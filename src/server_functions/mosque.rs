use leptos::{prelude::ServerFnError, *};
use surrealdb::sql::Geometry;
use surrealdb::RecordId;

use crate::{database::connection::get_db, models::{api_responses::ApiResponse, mosque::{Mosque, MosquesResponse}}};

#[server(prefix = "/mosque", endpoint = "fetch-through-region")]
pub async fn add_mosques_of_region(
    south: f64,
    west: f64,
    north: f64,
    east: f64,
) -> Result<ApiResponse<String>, ServerFnError> {

    let query = format!(
        r#"[out:json];
        (
            node["amenity"="place_of_worship"]["religion"="muslim"]["building"="mosque"]({},{},{},{});
            way["amenity"="place_of_worship"]["religion"="muslim"]["building"="mosque"]({},{},{},{});
        )
        out center;"#,
    south, west, north, east,
    south, west, north, east
    );

    let client = reqwest::Client::new();
    let response = client
        .post("https://overpass-api.de/api/interpreter")
        .body(query)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
        return Err(ServerFnError::ServerError(format!("Fetching data returned non 200 status, status: {}, response: {}", status, body)));
    }

    let data: MosquesResponse = response.json().await?;

    let mosques: Vec<Mosque> = data.elements
        .into_iter()
        .filter_map(|elem| {
            let (lat, lon) = match elem.element_type.as_str() {
                "node" => (elem.lat?, elem.lon?),
                "way" => {
                    let center = elem.center?;
                    (center.lat, center.lon)
                },
                _ => return None,
            };
            let location = Geometry::Point((lon, lat).into());
            let tags = elem.tags?;

            Some(Mosque {
                id: RecordId::from(("mosques", elem.id)),
                name: tags.name.unwrap_or_else(|| "Unnamed Mosque".to_string()),
                location,
                street: tags.street,
                city: tags.city,
            })
        }).collect();

    let db = get_db();
    db.create::<Option<Mosque>>("mosques")
        .content(mosques)
        .await?;

    Ok(ApiResponse {
        data: Some(format!("Added mosques for the region {} {} {} {}", south, west, north, east)),
        error: None,
    })
}
