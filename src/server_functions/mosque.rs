#[cfg(feature = "ssr")]
use crate::{
    errors::user_elevation::UserElevationError,
    utils::{
        mosque::enrich_mosques_with_contacts,
        parsing::parse_record_id,
        ssr::{ServerResponse, get_authenticated_user_and_context, get_server_context},
        user_elevation::elevate_user,
        user_elevation::is_mosque_admin,
    },
};
use leptos::{
    prelude::ServerFnError,
    server_fn::codec::{DeleteUrl, GetUrl, Json, PatchJson},
    *,
};

use crate::models::{
    api_responses::{ApiResponse, MixedMosqueResponse, MosqueResponse},
    mosque::PrayerTimesUpdate,
};

#[cfg(feature = "ssr")]
use crate::models::mosque::{
    MosqueFromOverpass, MosqueRecord, MosqueSearchResult, OverpassResponse,
};
#[cfg(feature = "ssr")]
use surrealdb::{RecordId, sql::Geometry};
#[cfg(feature = "ssr")]
use tracing::error;

#[server(input=Json, output=Json, prefix = "/mosques", endpoint = "add-mosque-of-region")]
pub async fn add_mosques_of_region(
    south: f64,
    west: f64,
    north: f64,
    east: f64,
) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    if !user.is_app_admin() && !user.is_mosque_supervisor() {
        error!(
            "Unauthorized attempt to add mosques of region by user {}",
            user.id
        );
        return Ok(responder.unauthorized("Only app admins can add mosques of region".to_string()));
    }

    let query = format!(
        r#"[out:json][timeout:30];
        (
            node["amenity"="place_of_worship"]["religion"="muslim"]({},{},{},{});
            way["amenity"="place_of_worship"]["religion"="muslim"]({},{},{},{});
        );
        out center;"#,
        south, west, north, east, south, west, north, east
    );

    let endpoints = [
        "https://overpass-api.de/api/interpreter",
        "https://overpass.kumi.systems/api/interpreter",
        "https://overpass.osm.ch/api/interpreter",
    ];

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()?;

    let mut response = None;
    let mut last_error = None;

    for endpoint in endpoints {
        let mut attempts = 0;
        let max_attempts = 2;

        while attempts < max_attempts {
            attempts += 1;
            match client.post(endpoint).body(query.clone()).send().await {
                Ok(res) => {
                    if res.status().is_success() {
                        response = Some(res);
                        break;
                    } else {
                        let status = res.status();
                        let body = res
                            .text()
                            .await
                            .unwrap_or_else(|_| "Could not read error body".to_string());
                        let err_msg =
                            format!("Endpoint {} returned {}, body: {}", endpoint, status, body);

                        error!("{}", err_msg);
                        last_error = Some(err_msg);
                        if status.is_server_error() && attempts < max_attempts {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            continue;
                        }
                        break; // Try next endpoint
                    }
                }
                Err(e) => {
                    let err_msg = format!("Endpoint {} failed: {}", endpoint, e);
                    error!("{}", err_msg);

                    last_error = Some(err_msg);
                    if attempts < max_attempts {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }
                    break; // Try next endpoint
                }
            }
        }

        if response.is_some() {
            break;
        }
    }

    let response = match response {
        Some(res) => res,
        None => {
            return Err(ServerFnError::ServerError(format!(
                "All Overpass API endpoints failed. Last error: {}",
                last_error.unwrap()
            )));
        }
    };
    let data: OverpassResponse = response.json().await?;

    let mosques: Vec<MosqueFromOverpass> = data
        .elements
        .into_iter()
        .filter_map(|elem| {
            let (lat, lon) = match elem.element_type.as_str() {
                "node" => (elem.lat?, elem.lon?),
                "way" => {
                    let center = elem.center?;
                    (center.lat, center.lon)
                }
                _ => return None,
            };
            let location = Geometry::Point((lon, lat).into());
            let (name, city, street, wikimedia_commons) = elem
                .tags
                .map(|tags| (tags.name, tags.street, tags.city, tags.wikimedia_commons))
                .unwrap_or((None, None, None, None));

            let cover_img = match wikimedia_commons {
                Some(ref filename) if !filename.is_empty() => Some(format!(
                    "https://commons.wikimedia.org/wiki/Special:FilePath/{}",
                    filename
                )),
                _ => None,
            };

            Some(MosqueFromOverpass {
                id: RecordId::from(("mosques", elem.id)),
                name,
                location,
                street,
                city,
                cover_img,
            })
        })
        .collect();

    let num_mosques = mosques.len();

    let insert_query = "INSERT INTO mosques $mosques";

    db.query(insert_query).bind(("mosques", mosques)).await?;

    Ok(ApiResponse {
        data: Some(format!(
            "Added {} mosques for the region {} {} {} {} successfully",
            num_mosques, south, west, north, east
        )),
        error: None,
    })
}

#[server(input = Json, output = Json, prefix = "/mosques", endpoint = "fetch-mosques-for-location")]
pub async fn fetch_mosques_for_location(
    lat: f64,
    lon: f64,
) -> Result<ApiResponse<Vec<MosqueResponse>>, ServerFnError> {
    let (_, db) = match get_server_context::<Vec<MosqueResponse>>().await {
        Ok(ctx) => ctx,
        Err(e) => {
            return Ok(ApiResponse {
                data: None,
                error: e.error,
            });
        }
    };
    let point = Geometry::Point((lon, lat).into());

    let radius_in_meters = 5000;
    let query = r#"
        SELECT *, geo::distance(location, $point) AS distance FROM mosques
        WHERE geo::distance(location, $point) < $radius
        ORDER BY distance ASC
        FETCH imam, muazzin
    "#;
    let mut response = db
        .query(query)
        .bind(("point", point))
        .bind(("radius", radius_in_meters))
        .await?;

    let mosques: Vec<MosqueSearchResult> = response.take(0)?;

    let mosque_responses = enrich_mosques_with_contacts(mosques, &db).await?;

    Ok(ApiResponse {
        data: Some(mosque_responses),
        error: None,
    })
}

#[server(input = PatchJson, output = Json, prefix = "/mosques", endpoint = "update-adhan-jamat-times")]
pub async fn update_adhan_jamat_times(
    mosque_id: String,
    prayer_times: PrayerTimesUpdate,
) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, mosque_admin) =
        match get_authenticated_user_and_context::<String>().await {
            Ok(ctx) => ctx,
            Err(e) => return Ok(e),
        };
    let responder = ServerResponse::new(response_options);

    let mosque_id: RecordId = match parse_record_id(&mosque_id, "mosque_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    if !mosque_admin.is_app_admin()
        && let Err(e) = is_mosque_admin(&mosque_admin.id, &mosque_id, &db).await
    {
        let msg = match e {
            UserElevationError::Unauthorized => {
                "The user trying to update mosque info is not an admin of that mosque".to_string()
            }
            _ => "Failed to verify admin permissions".to_string(),
        };
        error!("{}", msg);
        return Ok(responder.internal_server_error(msg));
    }

    db.update::<Option<MosqueRecord>>(mosque_id)
        .merge(prayer_times)
        .await?;

    Ok(responder.ok("Successfully updated jamat and adhan times".to_string()))
}

#[server(input = Json, output = Json, prefix = "/mosques", endpoint = "add-admin")]
pub async fn add_admin(
    requested_user: String,
    mosque_id: String,
) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, mosque_supervisor) =
        match get_authenticated_user_and_context::<String>().await {
            Ok(ctx) => ctx,
            Err(e) => return Ok(e),
        };
    let responder = ServerResponse::new(response_options);

    let requested_user: RecordId = match parse_record_id(&requested_user, "requested_user") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let mosque_id: RecordId = match parse_record_id(&mosque_id, "mosque_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    if !mosque_supervisor.is_mosque_supervisor() && !mosque_supervisor.is_app_admin() {
        error!(
            "The user {} trying to elevate other user's permission to mosque_admin is not a mosque_supervisor or app_admin",
            mosque_supervisor.id
        );
        return Ok(responder.unauthorized("The user trying to elevate other user's permission to mosque_admin is not a mosque_supervisor or app_admin".to_string()));
    }

    let relation_query = r#"
        RELATE $requested_user -> handles -> $mosque
            SET granted_by = $mosque_supervisor 
    "#;
    let elevation_result = db
        .query(relation_query)
        .bind(("requested_user", requested_user))
        .bind(("mosque", mosque_id))
        .bind(("mosque_supervisor", mosque_supervisor.id))
        .await;

    match elevation_result {
        Ok(_) => (),
        Err(error) => {
            error!(
                ?error,
                "Failed to elevate the user to a mosque admin due to db error"
            );
            return Err(ServerFnError::ServerError(
                "Failed to elevate the user to a mosque admin due to db error".to_string(),
            ));
        }
    }

    Ok(responder.ok("Elevated the user to a requested_user".to_string()))
}

#[server(input = Json, output = Json, prefix = "/mosques", endpoint = "elevate-user-to-mosque-supervisor")]
pub async fn elevate_user_to_mosque_supervisor(
    user_id: String,
) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, app_admin) =
        match get_authenticated_user_and_context::<String>().await {
            Ok(ctx) => ctx,
            Err(e) => return Ok(e),
        };
    let responder = ServerResponse::new(response_options);

    let user_id: RecordId = match parse_record_id(&user_id, "user_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let result = elevate_user(app_admin.id, user_id, "mosque_supervisor".to_string(), &db).await;

    match result {
        Ok(success_msg) => return Ok(responder.ok(success_msg)),
        Err(elevation_error) => match elevation_error {
            UserElevationError::Unauthorized => {
                return Ok(responder
                    .unauthorized("You are not authorized to perform this action".to_string()));
            }
            UserElevationError::AdminNotFound => {
                return Ok(responder.unauthorized("Admin user not found".to_string()));
            }
            UserElevationError::TargetUserNotFound => {
                return Ok(responder.not_found("User to elevate not found".to_string()));
            }
            UserElevationError::AlreadyElevated(role) => {
                return Ok(responder.conflict(format!("User is already a {}", role)));
            }
            UserElevationError::SelfElevationNotAllowed => {
                return Ok(responder.bad_request("You cannot elevate yourself".to_string()));
            }
            UserElevationError::DatabaseError(db_err) => {
                error!(?db_err, "Database error during user elevation");
                return Err(ServerFnError::ServerError(
                    "Internal server error during elevation".to_string(),
                ));
            }
        },
    }
}

#[server(input = GetUrl, output = Json, prefix = "/mosques", endpoint = "favorite")]
pub async fn get_favorite_mosque(
    lat: Option<f64>,
    lon: Option<f64>,
) -> Result<ApiResponse<MixedMosqueResponse>, ServerFnError> {
    let (response_options, db, user) =
        match get_authenticated_user_and_context::<MixedMosqueResponse>().await {
            Ok(ctx) => ctx,
            Err(e) => return Ok(e),
        };

    let responder = ServerResponse::new(response_options);
    let user_id = user.id;

    let favorite_mosques_query = r#"
       SELECT $user_id -> favorited -> mosques; 
    "#;

    let result = db
        .query(favorite_mosques_query)
        .bind(("user_id", user_id))
        .await;

    let favorite_mosques_result = match result {
        Ok(mut res) => res.take(0),
        Err(e) => {
            error!(
                ?e,
                "Some db error occured while quering for the user's favorite mosques"
            );
            return Ok(responder.internal_server_error::<MixedMosqueResponse>(
                "DB error occured while getting the favorite mosques of the user".to_string(),
            ));
        }
    };

    let favorite_mosques: Vec<MosqueSearchResult> = match favorite_mosques_result {
        Ok(mosques) => mosques,
        Err(e) => {
            error!(?e, "No favorite mosques for the user have been found");
            return Ok(responder.not_found::<MixedMosqueResponse>(
                "The user doesn't have any favorite mosques".to_string(),
            ));
        }
    };

    let favorite_mosques_response_result =
        enrich_mosques_with_contacts(favorite_mosques, &db).await;

    let favorite_mosques_response: Vec<MosqueResponse> = match favorite_mosques_response_result {
        Ok(mosques) => mosques,
        Err(e) => {
            error!(?e, "Unable to enrichthe mosques with contacts");
            return Ok(responder.internal_server_error::<MixedMosqueResponse>(
                "Unable to create enriched mosque data with contacts".to_string(),
            ));
        }
    };

    return Ok(responder
        .ok::<MixedMosqueResponse>(MixedMosqueResponse::MosquesVec(favorite_mosques_response)));
}

#[server(input = Json, output = Json, prefix = "/mosques", endpoint = "add-favorite")]
pub async fn add_favorite(mosque_id: String) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let mosque_id = match parse_record_id(&mosque_id, "mosque_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let favorite_query = r#"
        RELATE $user_id -> favorited -> $mosque_id;
        "#;

    let result = db
        .query(favorite_query)
        .bind(("user_id", user.id))
        .bind(("mosque_id", mosque_id))
        .await;

    match result {
        Ok(_) => (),
        Err(e) => {
            error!(?e, "Database error");
            return Ok(responder.internal_server_error("Failed to favorite a mosque".to_string()));
        }
    }

    Ok(responder.ok("Successfully added the mosque to user's favorite list".to_string()))
}

#[server(input = DeleteUrl, output = Json, prefix = "/mosques", endpoint = "/remove-favorite")]
pub async fn remove_favorite(mosque_id: String) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let mosque_id = match parse_record_id(&mosque_id, "mosque_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let remove_favorite_query = "DELETE favorited WHERE in = $user_id AND out = $mosque_id";

    let result = db
        .query(remove_favorite_query)
        .bind(("user_id", user.id))
        .bind(("mosque_id", mosque_id))
        .await;

    match result {
        Ok(_) => (),
        Err(e) => {
            error!(?e, "Failed to remove favorited mosque for the user");
            return Ok(responder.internal_server_error(
                "Failed to remove favorited mosque for the user".to_string(),
            ));
        }
    }

    Ok(responder.ok("Successfully removed the mosque from favorite list of the user".to_string()))
}

#[server(input = PatchJson, output = Json, prefix = "/mosques", endpoint = "update-personnel")]
pub async fn update_mosque_personnel(
    person_type: String,
    person_id: String,
    mosque_id: String,
) -> Result<ApiResponse, ServerFnError> {
    let (response_options, db, auth_user) =
        match get_authenticated_user_and_context::<String>().await {
            Ok(ctx) => ctx,
            Err(e) => return Ok(e),
        };
    let responder = ServerResponse::new(response_options);

    if person_type != "imam" && person_type != "muazzin" {
        return Ok(
            responder.bad_request("person_type must be either 'imam' or 'muazzin'".to_string())
        );
    }

    let person_id: RecordId = match parse_record_id(&person_id, "person_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let mosque_id: RecordId = match parse_record_id(&mosque_id, "mosque_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    if !auth_user.is_app_admin()
        && let Err(e) = is_mosque_admin(&auth_user.id, &mosque_id, &db).await
    {
        let msg = match e {
            UserElevationError::Unauthorized => {
                "The user trying to update mosque info is not an admin of that mosque".to_string()
            }
            _ => "Failed to verify admin permissions".to_string(),
        };
        error!("{}", msg);
        return Ok(responder.internal_server_error(msg));
    }

    let update_query = format!(
        "UPDATE mosques SET {} = $person_id WHERE id = $mosque_id",
        person_type
    );
    let result = db
        .query(update_query)
        .bind(("person_id", person_id))
        .bind(("mosque_id", mosque_id))
        .await;

    match result {
        Ok(_) => Ok(responder.ok(format!(
            "Successfully updated mosque {} information",
            person_type
        ))),
        Err(e) => {
            error!(?e, "Failed to update mosque personnel");
            Ok(responder.internal_server_error(
                "Failed to update mosque personnel due to database error".to_string(),
            ))
        }
    }
}
