use leptos::{prelude::ServerFnError, *};
use crate::models::{api_responses::ApiResponse, mosque::{Mosque, MosquesResponse}};
#[cfg(feature = "ssr")]
use surrealdb::sql::Geometry;
#[cfg(feature = "ssr")]
use surrealdb::RecordId;

#[cfg(feature = "ssr")]
use crate::database::connection::get_db;

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
        data: Some(format!("Added mosques for the region {} {} {} {} successfully", south, west, north, east)),
        error: None,
    })
}

#[server(prefix = "/mosque", endpoint = "fetch-mosques-from-region")]
pub async fn fetch_mosques_from_region(lat: f64, lon: f64) -> Result<ApiResponse<Vec<Mosque>>, ServerFnError> {
    let db = get_db();
    let point = Geometry::Point((lon, lat).into());
    
    let radius_in_meters = 10000;
    let query = r#"
        SELECT * FROM mosques
        WHERE geo::distance(location, $point) < $radius
        ORDER BY geo::distance(location, $point) ASC
    "#;
    let mut result = db.query(query)
        .bind(("point", point))
        .bind(("radius", radius_in_meters))
        .await?;

    let mosques: Vec<Mosque> = result.take(0)?;

    Ok(ApiResponse {
        data: Some(mosques),
        error: None,
    })
}
