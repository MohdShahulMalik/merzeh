use crate::common::get_test_db;
use merzah::auth::custom_auth::register_user;
use merzah::auth::session::create_session;
use merzah::{
    models::{
        api_responses::ApiResponse,
        auth::{Platform, RegistrationFormData},
        mosque::{MosqueFromOverpass, MosqueSearchResult},
        user::{Identifier, User},
    },
    spawn_app,
};
use reqwest::Client;
use rstest::rstest;
use serde::{Deserialize, Serialize};
use surrealdb::{RecordId, Surreal, engine::remote::ws::Client as SurrealClient, sql::Geometry};

#[derive(Serialize)]
struct AddAdminPayload {
    mosque_supervisor: String,
    requested_user: String,
    mosque_id: String,
}

#[derive(Serialize)]
struct Role {
    role: String,
}

#[derive(serde::Deserialize, Serialize)]
struct Handle {
    granted_by: RecordId,
}

async fn create_user(
    db: &Surreal<SurrealClient>,
    name: &str,
    email: &str,
    role: Option<&str>,
) -> (User, String) {
    let unique_email = format!("{}_{}", uuid::Uuid::new_v4(), email);
    let form = RegistrationFormData::new(
        name.to_string(),
        Identifier::Email(unique_email),
        "password".to_string(),
        Platform::Web,
    );
    let user_id = register_user(form, db)
        .await
        .expect("Failed to register user");

    if let Some(r) = role {
        let _: Option<User> = db
            .update(user_id.clone())
            .merge(Role {
                role: r.to_string(),
            })
            .await
            .expect("Failed to set role");
    }

    let user = db
        .select(user_id.clone())
        .await
        .expect("User not found")
        .unwrap();
    let session_token = create_session(user_id, db)
        .await
        .expect("Failed to create session");

    (user, session_token)
}

#[rstest]
#[case::is_supervisor("mosque_supervisor", true, None)]
#[case::not_supervisor("regular", false, Some("not a mosque_supervisor"))]
#[tokio::test]
async fn test_add_admin_endpoint(
    #[case] supervisor_role: &str,
    #[case] should_succeed: bool,
    #[case] expected_error_part: Option<&str>,
) {
    use surrealdb::RecordId;

    let db = get_test_db().await;
    let addr = spawn_app(db.clone());
    let client = Client::new();
    let url = format!("{}/mosques/add-admin", addr);

    // Setup Data
    let (supervisor, supervisor_session) =
        create_user(&db, "Supervisor", "super@test.com", Some(supervisor_role)).await;
    let (new_admin, _) = create_user(&db, "New Admin", "admin@test.com", Some("regular")).await;

    // Create a dummy mosque
    let _: Option<MosqueSearchResult> = db
        .create("mosques")
        .content(MosqueFromOverpass {
            id: RecordId::from(("mosque", "test_mosque_1")),
            name: Some("test_mosque_1".to_string()),
            location: Geometry::Point((9.00, 8.00).into()),
            city: None,
            street: None,
            cover_img: None,
        })
        .await
        .expect("failed to create a new mosque");

    let mosque_id = "mosques:test_mosque_1";
    let payload = AddAdminPayload {
        mosque_supervisor: supervisor.id.to_string(),
        requested_user: new_admin.id.to_string(),
        mosque_id: mosque_id.to_string(),
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", supervisor_session))
        .json(&payload)
        .send()
        .await
        .expect("Failed to send request");

    // We need to parse the ApiResponse wrapper
    let api_response: ApiResponse<String> = response
        .json()
        .await
        .expect("Failed to deserialize response");

    if should_succeed {
        assert!(
            api_response.error.is_none(),
            "Expected success but got error: {:?}",
            api_response.error
        );
        assert_eq!(
            api_response.data,
            Some("Elevated the user to a requested_user".to_string())
        );

        // Verify Relation in DB
        let relation_query = "SELECT * FROM handles WHERE in = $user AND out = $mosque";
        let mut res = db
            .query(relation_query)
            .bind(("user", new_admin.id.clone()))
            .bind((
                "mosque",
                surrealdb::RecordId::from(("mosques", "test_mosque_1")),
            ))
            .await
            .expect("Query failed");

        let relations: Vec<Handle> = res.take(0).unwrap();
        assert!(!relations.is_empty(), "Relation 'handles' was not created");
        assert_eq!(
            relations[0].granted_by.to_string(),
            supervisor.id.to_string().replace("⟨", "").replace("⟩", "")
        ); // Basic check, ID format might vary
    } else {
        // The API returns OK(200) with error message in body for some logic errors (based on code analysis)
        // OR it returns 401 Unauthorized for the supervisor check.
        // Let's check the logic:
        // logic: `response_options.set_status(StatusCode::UNAUTHORIZED); return Ok(ApiResponse::error(...))`
        // So status might be 401, but body is still JSON ApiResponse.

        assert!(
            api_response.error.is_some(),
            "Expected error but got success"
        );
        let err_msg = api_response.error.unwrap();
        if let Some(part) = expected_error_part {
            assert!(
                err_msg.contains(part),
                "Error '{}' did not contain '{}'",
                err_msg,
                part
            );
        }
    }
}
