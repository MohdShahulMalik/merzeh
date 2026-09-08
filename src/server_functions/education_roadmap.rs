use leptos::{prelude::ServerFnError, server_fn::codec::Json, *};
#[cfg(feature = "ssr")]
use serde::Deserialize;
#[cfg(feature = "ssr")]
use surrealdb::RecordId;

use crate::models::api_responses::ApiResponse;
#[cfg(feature = "ssr")]
use crate::models::education::Course;
#[cfg(feature = "ssr")]
use crate::models::roadmap::RoadmapStatus;
#[cfg(feature = "ssr")]
use crate::models::roadmap::{Framework, Milestone, Roadmap};
use crate::models::roadmap::{
    FrameworkDetail, FrameworkOnClient, MilestoneOnClient, RoadmapCourseOnClient, RoadmapDetail,
    RoadmapOnClient,
};
#[cfg(feature = "ssr")]
use crate::services::course_stats::update_course_enrollment_count;
#[cfg(feature = "ssr")]
use crate::utils::parsing::parse_record_id;
#[cfg(feature = "ssr")]
use crate::utils::ssr::{ServerResponse, get_authenticated_user_and_context, get_server_context};

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct RoadmapCourseWithCourse {
    pub out: Course,
    pub sort_order: i32,
    pub is_required: bool,
    pub note: Option<String>,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct MilestoneCourseWithCourse {
    pub out: Course,
    pub is_required: bool,
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "roadmaps")]
pub async fn fetch_roadmaps() -> Result<ApiResponse<Vec<RoadmapOnClient>>, ServerFnError> {
    let (response_options, db) = match get_server_context::<Vec<RoadmapOnClient>>().await {
        Ok(ctx) => ctx,
        Err(e) => {
            return Ok(ApiResponse {
                data: None,
                error: e.error,
            });
        }
    };
    let responder = ServerResponse::new(response_options);

    let mut response = db
        .query("SELECT * FROM roadmaps WHERE status = \"published\" AND deleted = false ORDER BY created_at DESC")
        .await?;
    let roadmaps: Vec<Roadmap> = response.take(0)?;

    let payload = roadmaps
        .into_iter()
        .map(|roadmap| RoadmapOnClient {
            id: roadmap.id.to_string(),
            title: roadmap.title,
            slug: roadmap.slug,
            description: roadmap.description,
            image_url: roadmap.image_url,
            difficulty: roadmap.difficulty,
            estimated_weeks: roadmap.estimated_weeks,
        })
        .collect::<Vec<_>>();

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "roadmap-detail")]
pub async fn fetch_roadmap_detail(
    roadmap_id: String,
) -> Result<ApiResponse<RoadmapDetail>, ServerFnError> {
    let (response_options, db) = match get_server_context::<RoadmapDetail>().await {
        Ok(ctx) => ctx,
        Err(e) => {
            return Ok(ApiResponse {
                data: None,
                error: e.error,
            });
        }
    };
    let responder = ServerResponse::new(response_options);

    let roadmap_id: RecordId = match parse_record_id(&roadmap_id, "roadmap_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let roadmap: Option<Roadmap> = db.select(roadmap_id.clone()).await?;
    let roadmap = match roadmap {
        Some(roadmap) if roadmap.status == RoadmapStatus::Published && !roadmap.deleted => roadmap,
        _ => return Ok(responder.not_found("Roadmap not found".to_string())),
    };

    let mut courses_response = db
        .query("SELECT out, sort_order, is_required, note FROM roadmap_courses WHERE in = $roadmap_id ORDER BY sort_order ASC FETCH out")
        .bind(("roadmap_id", roadmap_id))
        .await?;
    let courses: Vec<RoadmapCourseWithCourse> = courses_response.take(0)?;

    let courses_payload = courses
        .into_iter()
        .map(|edge| RoadmapCourseOnClient {
            course_id: edge.out.id.to_string(),
            title: edge.out.title,
            short_description: edge.out.short_description,
            thumbnail_url: edge.out.thumbnail_url,
            sort_order: edge.sort_order,
            is_required: edge.is_required,
            note: edge.note,
        })
        .collect::<Vec<_>>();

    let payload = RoadmapDetail {
        id: roadmap.id.to_string(),
        title: roadmap.title,
        slug: roadmap.slug,
        description: roadmap.description,
        image_url: roadmap.image_url,
        difficulty: roadmap.difficulty,
        estimated_weeks: roadmap.estimated_weeks,
        courses: courses_payload,
    };

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "roadmap-start")]
pub async fn start_roadmap(roadmap_id: String) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let roadmap_id: RecordId = match parse_record_id(&roadmap_id, "roadmap_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let roadmap: Option<Roadmap> = db.select(roadmap_id.clone()).await?;
    let _roadmap = match roadmap {
        Some(roadmap) if roadmap.status == RoadmapStatus::Published && !roadmap.deleted => roadmap,
        _ => return Ok(responder.not_found("Roadmap not found".to_string())),
    };

    let mut courses_response = db
        .query("SELECT out, is_required FROM roadmap_courses WHERE in = $roadmap_id ORDER BY sort_order ASC FETCH out")
        .bind(("roadmap_id", roadmap_id))
        .await?;
    let courses: Vec<RoadmapCourseWithCourse> = courses_response.take(0)?;

    for course in courses {
        if !course.is_required {
            continue;
        }
        let enroll_query = "RELATE $user_id -> enrolled -> $course_id SET enrolled_at = time::now(), progress_percent = 0, last_accessed_at = time::now()";
        let enroll_result = db
            .query(enroll_query)
            .bind(("user_id", user.id.clone()))
            .bind(("course_id", course.out.id.clone()))
            .await;
        if enroll_result.is_ok() {
            let _ = update_course_enrollment_count(&course.out.id, &db).await;
        }
    }

    Ok(responder.ok("Roadmap started".to_string()))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "frameworks")]
pub async fn fetch_frameworks() -> Result<ApiResponse<Vec<FrameworkOnClient>>, ServerFnError> {
    let (response_options, db) = match get_server_context::<Vec<FrameworkOnClient>>().await {
        Ok(ctx) => ctx,
        Err(e) => {
            return Ok(ApiResponse {
                data: None,
                error: e.error,
            });
        }
    };
    let responder = ServerResponse::new(response_options);

    let mut response = db
        .query("SELECT * FROM frameworks WHERE status = \"published\" AND deleted = false ORDER BY created_at DESC")
        .await?;
    let frameworks: Vec<Framework> = response.take(0)?;

    let payload = frameworks
        .into_iter()
        .map(|framework| FrameworkOnClient {
            id: framework.id.to_string(),
            title: framework.title,
            slug: framework.slug,
            description: framework.description,
            image_url: framework.image_url,
        })
        .collect::<Vec<_>>();

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "framework-detail")]
pub async fn fetch_framework_detail(
    framework_id: String,
) -> Result<ApiResponse<FrameworkDetail>, ServerFnError> {
    let (response_options, db) = match get_server_context::<FrameworkDetail>().await {
        Ok(ctx) => ctx,
        Err(e) => {
            return Ok(ApiResponse {
                data: None,
                error: e.error,
            });
        }
    };
    let responder = ServerResponse::new(response_options);

    let framework_id: RecordId = match parse_record_id(&framework_id, "framework_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let framework: Option<Framework> = db.select(framework_id.clone()).await?;
    let framework = match framework {
        Some(framework) if framework.status == "published" && !framework.deleted => framework,
        _ => return Ok(responder.not_found("Framework not found".to_string())),
    };

    let mut milestones_response = db
        .query("SELECT * FROM milestones WHERE framework = $framework_id ORDER BY sort_order ASC")
        .bind(("framework_id", framework_id))
        .await?;
    let milestones: Vec<Milestone> = milestones_response.take(0)?;

    let mut milestone_payload = Vec::new();
    for milestone in milestones {
        let mut courses_response = db
            .query(
                "SELECT out, is_required FROM milestone_courses WHERE in = $milestone_id FETCH out",
            )
            .bind(("milestone_id", milestone.id.clone()))
            .await?;
        let courses: Vec<MilestoneCourseWithCourse> = courses_response.take(0)?;

        let courses_payload = courses
            .into_iter()
            .map(|edge| RoadmapCourseOnClient {
                course_id: edge.out.id.to_string(),
                title: edge.out.title,
                short_description: edge.out.short_description,
                thumbnail_url: edge.out.thumbnail_url,
                sort_order: 0,
                is_required: edge.is_required,
                note: None,
            })
            .collect::<Vec<_>>();

        milestone_payload.push(MilestoneOnClient {
            id: milestone.id.to_string(),
            title: milestone.title,
            description: milestone.description,
            sort_order: milestone.sort_order,
            courses: courses_payload,
        });
    }

    let payload = FrameworkDetail {
        id: framework.id.to_string(),
        title: framework.title,
        slug: framework.slug,
        description: framework.description,
        image_url: framework.image_url,
        milestones: milestone_payload,
    };

    Ok(responder.ok(payload))
}
