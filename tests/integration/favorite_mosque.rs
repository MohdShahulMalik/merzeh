use crate::common::get_test_db;
use merzah::auth::session::create_session;
use merzah::models::api_responses::ApiResponse;
use merzah::models::api_responses::MixedMosqueResponse;
use merzah::models::mosque::MosqueRecord;
use merzah::models::user::User;
use merzah::spawn_app;
use reqwest::Client;
use rstest::rstest;
use serde::Serialize;
use surrealdb::Datetime;
use surrealdb::RecordId;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client as WsClient;
use surrealdb::sql::Geometry;

#[derive(Serialize)]
struct CreateMosque {
    location: Geometry,
    name: String,
}

#[derive(Serialize)]
struct FavoriteMosqueQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lon: Option<f64>,
}

async fn setup_user(db: &Surreal<WsClient>) -> (User, String) {
    let user: User = db
        .create("users")
        .content(User {
            id: RecordId::from(("users", format!("fav_user_{}", uuid::Uuid::new_v4()))),
            created_at: Datetime::default(),
            display_name: "Favorite User".to_string(),
            password_hash: "hash".to_string(),
            role: "regular".to_string(),
            updated_at: Datetime::default(),
        })
        .await
        .expect("Failed to create user")
        .expect("User not returned");

    let session = create_session(user.id.clone(), db)
        .await
        .expect("Failed to create session");

    (user, session)
}

async fn relate_favorite(db: &Surreal<WsClient>, user: &User, mosque_id: &RecordId) {
    db.query("RELATE $user -> favorited -> $mosque")
        .bind(("user", user.id.clone()))
        .bind(("mosque", mosque_id.clone()))
        .await
        .expect("Failed to relate favorite");
}

async fn create_mosque(db: &Surreal<WsClient>, lon: f64, lat: f64, name: &str) -> MosqueRecord {
    db.create("mosques")
        .content(CreateMosque {
            location: Geometry::Point((lon, lat).into()),
            name: name.to_string(),
        })
        .await
        .expect("Failed to create mosque")
        .expect("Mosque not returned")
}

async fn setup_user_with_two_favorites(db: &Surreal<WsClient>) -> (String, RecordId, RecordId) {
    let near = create_mosque(db, 77.295, 28.625, "Near Mosque").await;
    let far = create_mosque(db, 77.305, 28.635, "Far Mosque").await;

    let (user, session) = setup_user(db).await;
    relate_favorite(db, &user, &near.id).await;
    relate_favorite(db, &user, &far.id).await;

    (session, near.id, far.id)
}

#[tokio::test]
async fn fetch_all_favorite_mosques() {
    let db = get_test_db().await;
    let addr = spawn_app(db.clone());
    let client = Client::new();
    let (session, near_id, far_id) = setup_user_with_two_favorites(&db).await;

    let fetch_url = format!("{}/mosques/favorite", addr);

    let response = client
        .get(&fetch_url)
        .header("Authorization", format!("Bearer {}", session))
        .send()
        .await
        .expect("Failed to fetch favorites");

    assert_eq!(response.status().as_u16(), 200);

    let api_response = response
        .json::<ApiResponse<MixedMosqueResponse>>()
        .await
        .expect("Failed to deserialize");

    match api_response.data.expect("No data returned") {
        MixedMosqueResponse::MosquesVec(mosques) => {
            assert_eq!(mosques.len(), 2, "Should return both favorites");
            let ids: Vec<String> = mosques.iter().map(|m| m.id.clone()).collect();
            assert!(
                ids.contains(&near_id.to_string()),
                "Should contain the near mosque"
            );
            assert!(
                ids.contains(&far_id.to_string()),
                "Should contain the far mosque"
            );
        }
        MixedMosqueResponse::SingleMosque(_) => {
            panic!("Expected MosquesVec when no coordinates are given, got SingleMosque")
        }
    }
}

#[tokio::test]
async fn fetch_all_favorites_with_no_favorites_returns_empty_vec() {
    let db = get_test_db().await;
    let addr = spawn_app(db.clone());
    let client = Client::new();
    let (_, session) = setup_user(&db).await;

    let fetch_url = format!("{}/mosques/favorite", addr);

    let response = client
        .get(&fetch_url)
        .header("Authorization", format!("Bearer {}", session))
        .send()
        .await
        .expect("Failed to fetch favorites");

    assert_eq!(response.status().as_u16(), 200);

    let api_response = response
        .json::<ApiResponse<MixedMosqueResponse>>()
        .await
        .expect("Failed to deserialize");

    match api_response.data.expect("No data returned") {
        MixedMosqueResponse::MosquesVec(mosques) => {
            assert!(mosques.is_empty(), "Should return an empty vec");
        }
        MixedMosqueResponse::SingleMosque(_) => {
            panic!("Expected MosquesVec when no coordinates are given, got SingleMosque")
        }
    }
}

#[rstest]
#[case(28.625, 77.295, true)]
#[case(28.635, 77.305, false)]
#[tokio::test]
async fn fetch_closest_favorite_mosque(
    #[case] lat: f64,
    #[case] lon: f64,
    #[case] expect_near: bool,
) {
    let db = get_test_db().await;
    let addr = spawn_app(db.clone());
    let client = Client::new();
    let (session, near_id, far_id) = setup_user_with_two_favorites(&db).await;

    let fetch_url = format!("{}/mosques/favorite", addr);
    let params = FavoriteMosqueQuery {
        lat: Some(lat),
        lon: Some(lon),
    };

    let response = client
        .get(&fetch_url)
        .query(&params)
        .header("Authorization", format!("Bearer {}", session))
        .send()
        .await
        .expect("Failed to fetch closest favorite");

    assert_eq!(response.status().as_u16(), 200);

    let api_response = response
        .json::<ApiResponse<MixedMosqueResponse>>()
        .await
        .expect("Failed to deserialize");

    match api_response.data.expect("No data returned") {
        MixedMosqueResponse::SingleMosque(mosque) => {
            let expected_id = if expect_near { near_id } else { far_id };
            assert_eq!(
                mosque.id,
                expected_id.to_string(),
                "Should return the closest favorited mosque"
            );
        }
        MixedMosqueResponse::MosquesVec(_) => {
            panic!("Expected SingleMosque when coordinates are given, got MosquesVec")
        }
    }
}

#[rstest]
#[case(Some(28.625), None)]
#[case(None, Some(77.295))]
#[tokio::test]
async fn fetch_favorite_mosque_with_partial_coordinates_is_bad_request(
    #[case] lat: Option<f64>,
    #[case] lon: Option<f64>,
) {
    let db = get_test_db().await;
    let addr = spawn_app(db.clone());
    let client = Client::new();
    let (session, _, _) = setup_user_with_two_favorites(&db).await;

    let fetch_url = format!("{}/mosques/favorite", addr);
    let params = FavoriteMosqueQuery { lat, lon };

    let response = client
        .get(&fetch_url)
        .query(&params)
        .header("Authorization", format!("Bearer {}", session))
        .send()
        .await
        .expect("Failed to fetch favorite");

    assert_eq!(response.status().as_u16(), 400);

    let api_response = response
        .json::<ApiResponse<MixedMosqueResponse>>()
        .await
        .expect("Failed to deserialize");

    assert!(api_response.data.is_none());
    assert_eq!(
        api_response.error,
        Some(
            "Both latitude and longitude must be provided to find the closest favorite mosque"
                .to_string()
        )
    );
}

#[tokio::test]
async fn fetch_closest_favorite_with_no_favorites_is_not_found() {
    let db = get_test_db().await;
    let addr = spawn_app(db.clone());
    let client = Client::new();
    let (_, session) = setup_user(&db).await;

    let fetch_url = format!("{}/mosques/favorite", addr);
    let params = FavoriteMosqueQuery {
        lat: Some(28.625),
        lon: Some(77.295),
    };

    let response = client
        .get(&fetch_url)
        .query(&params)
        .header("Authorization", format!("Bearer {}", session))
        .send()
        .await
        .expect("Failed to fetch closest favorite");

    assert_eq!(response.status().as_u16(), 404);

    let api_response = response
        .json::<ApiResponse<MixedMosqueResponse>>()
        .await
        .expect("Failed to deserialize");

    assert!(api_response.data.is_none());
    assert_eq!(
        api_response.error,
        Some("The user doesn't have any favorite mosques".to_string())
    );
}

#[tokio::test]
async fn fetch_favorite_mosque_unauthenticated_is_unauthorized() {
    let db = get_test_db().await;
    let addr = spawn_app(db.clone());
    let client = Client::new();

    let fetch_url = format!("{}/mosques/favorite", addr);

    let response = client
        .get(&fetch_url)
        .header("Authorization", "Bearer invalid_token")
        .send()
        .await
        .expect("Failed to fetch favorite");

    assert_eq!(response.status().as_u16(), 401);
}
