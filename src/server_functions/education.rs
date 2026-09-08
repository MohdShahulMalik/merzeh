#[cfg(feature = "ssr")]
use chrono::{DateTime, FixedOffset, Utc};
#[cfg(feature = "ssr")]
use garde::Validate;
use leptos::{
    prelude::ServerFnError,
    server_fn::codec::{DeleteUrl, Json, PatchJson},
    *,
};
#[cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::{Datetime, RecordId};
#[cfg(feature = "ssr")]
use tracing::error;

use crate::models::api_responses::ApiResponse;
#[cfg(feature = "ssr")]
use crate::models::education::{
    Course, CourseRecord, Lesson, LessonRecord, Module, ModuleRecord, Track, UpdatedCourseRecord,
    UpdatedLessonRecord, UpdatedModuleRecord,
};
use crate::models::education::{
    CourseDetail, CourseLevel, CourseOnClient, CourseStatus, CreateCourse as CreateCourseInput,
    CreateLesson as CreateLessonInput, CreateModule as CreateModuleInput, EducatorInfo,
    EnrollmentProgress, LessonDetail, LessonOnClient, ModuleWithLessons, TrackOnClient,
    UpdateCourse as UpdateCourseInput, UpdateLesson as UpdateLessonInput,
    UpdateModule as UpdateModuleInput,
};
#[cfg(feature = "ssr")]
use crate::models::user::User;
#[cfg(feature = "ssr")]
use crate::services::achievement::check_and_award_achievements;
#[cfg(feature = "ssr")]
use crate::services::course_stats::{update_course_enrollment_count, update_course_lesson_count};
#[cfg(feature = "ssr")]
use crate::services::streak::bump_streak;
#[cfg(feature = "ssr")]
use crate::utils::education_auth::{is_course_owner, is_educator_or_admin};
#[cfg(feature = "ssr")]
use crate::utils::parsing::parse_record_id;
#[cfg(feature = "ssr")]
use crate::utils::ssr::{ServerResponse, get_authenticated_user_and_context, get_server_context};
#[cfg(feature = "ssr")]
use crate::utils::token_generator::generate_token;

#[cfg(feature = "ssr")]
fn datetime_to_fixed(datetime: Datetime) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(&datetime.to_string())
        .unwrap_or_else(|_| Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()))
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct CountResult {
    pub count: i64,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct CourseWithEducator {
    pub id: RecordId,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub short_description: String,
    pub level: CourseLevel,
    pub thumbnail_url: Option<String>,
    pub video_url: Option<String>,
    pub duration_minutes: i32,
    pub lesson_count: i32,
    pub enrollment_count: i32,
    pub educator: User,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct EnrolledWithCourse {
    pub out: Course,
    pub enrolled_at: Datetime,
    pub progress_percent: f32,
    pub last_accessed_at: Option<Datetime>,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct RelationId {
    pub id: RecordId,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct EnrolledWithLessonId {
    pub out: RecordId,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Serialize)]
struct EnrollmentUpdate {
    pub progress_percent: f32,
    pub last_accessed_at: Datetime,
    pub completed_at: Option<Datetime>,
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "tracks")]
pub async fn fetch_tracks() -> Result<ApiResponse<Vec<TrackOnClient>>, ServerFnError> {
    let (response_options, db) = match get_server_context::<Vec<TrackOnClient>>().await {
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
        .query("SELECT * FROM tracks WHERE deleted = false ORDER BY sort_order ASC")
        .await?;

    let tracks: Vec<Track> = response.take(0)?;
    let mut payload = Vec::new();

    for track in tracks {
        let mut count_response = db
            .query(
                "SELECT count() AS count FROM courses WHERE track = $track AND status = \"published\" AND deleted = false",
            )
            .bind(("track", track.id.clone()))
            .await?;
        let counts: Vec<CountResult> = count_response.take(0)?;
        let count = counts.first().map(|c| c.count).unwrap_or(0) as usize;

        payload.push(TrackOnClient {
            id: track.id.to_string(),
            name: track.name,
            slug: track.slug,
            description: track.description,
            icon: track.icon,
            image_url: track.image_url,
            course_count: count,
        });
    }

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "track-courses")]
pub async fn fetch_track_courses(
    track_id: String,
) -> Result<ApiResponse<Vec<CourseOnClient>>, ServerFnError> {
    let (response_options, db) = match get_server_context::<Vec<CourseOnClient>>().await {
        Ok(ctx) => ctx,
        Err(e) => {
            return Ok(ApiResponse {
                data: None,
                error: e.error,
            });
        }
    };
    let responder = ServerResponse::new(response_options);

    let track_id: RecordId = match parse_record_id(&track_id, "track_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let mut response = db
        .query(
            "SELECT id, title, slug, description, short_description, level, thumbnail_url, video_url, duration_minutes, lesson_count, enrollment_count, educator FROM courses WHERE track = $track_id AND status = \"published\" AND deleted = false FETCH educator",
        )
        .bind(("track_id", track_id))
        .await?;

    let courses: Vec<CourseWithEducator> = response.take(0)?;
    let payload = courses
        .into_iter()
        .map(|course| CourseOnClient {
            id: course.id.to_string(),
            title: course.title,
            slug: course.slug,
            short_description: course.short_description,
            level: course.level,
            thumbnail_url: course.thumbnail_url,
            video_url: course.video_url,
            duration_minutes: course.duration_minutes,
            lesson_count: course.lesson_count,
            enrollment_count: course.enrollment_count,
            educator_name: course.educator.display_name,
        })
        .collect::<Vec<_>>();

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "course")]
pub async fn fetch_course_details(
    course_id: String,
) -> Result<ApiResponse<CourseDetail>, ServerFnError> {
    let (response_options, db) = match get_server_context::<CourseDetail>().await {
        Ok(ctx) => ctx,
        Err(e) => {
            return Ok(ApiResponse {
                data: None,
                error: e.error,
            });
        }
    };
    let responder = ServerResponse::new(response_options);

    let course_id: RecordId = match parse_record_id(&course_id, "course_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let mut response = db
        .query(
            "SELECT * FROM courses WHERE id = $course_id AND status = \"published\" AND deleted = false FETCH educator",
        )
        .bind(("course_id", course_id.clone()))
        .await?;

    let course: Option<CourseWithEducator> = response.take(0)?;
    let course = match course {
        Some(course) => course,
        None => return Ok(responder.not_found("Course not found".to_string())),
    };

    let mut module_response = db
        .query("SELECT * FROM modules WHERE course = $course_id AND deleted = false ORDER BY sort_order ASC")
        .bind(("course_id", course_id.clone()))
        .await?;
    let modules: Vec<Module> = module_response.take(0)?;

    let mut module_payload = Vec::new();
    for module in modules {
        let mut lesson_response = db
            .query("SELECT * FROM lessons WHERE module = $module_id AND deleted = false ORDER BY sort_order ASC")
            .bind(("module_id", module.id.clone()))
            .await?;
        let lessons: Vec<Lesson> = lesson_response.take(0)?;

        let lessons_payload = lessons
            .into_iter()
            .map(|lesson| LessonOnClient {
                id: lesson.id.to_string(),
                title: lesson.title,
                content_type: lesson.content_type,
                thumbnail_url: lesson.thumbnail_url,
                duration_minutes: lesson.duration_minutes,
                sort_order: lesson.sort_order,
                is_preview: lesson.is_preview,
                is_completed: false,
            })
            .collect::<Vec<_>>();

        module_payload.push(ModuleWithLessons {
            id: module.id.to_string(),
            title: module.title,
            description: module.description,
            sort_order: module.sort_order,
            lessons: lessons_payload,
        });
    }

    let payload = CourseDetail {
        id: course.id.to_string(),
        title: course.title,
        slug: course.slug,
        description: course.description,
        short_description: course.short_description,
        level: course.level,
        thumbnail_url: course.thumbnail_url,
        video_url: course.video_url,
        duration_minutes: course.duration_minutes,
        lesson_count: course.lesson_count,
        enrollment_count: course.enrollment_count,
        educator: EducatorInfo {
            id: course.educator.id.to_string(),
            display_name: course.educator.display_name,
        },
        modules: module_payload,
        is_enrolled: false,
        progress_percent: 0.0,
    };

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "lesson")]
pub async fn fetch_lesson_details(
    lesson_id: String,
) -> Result<ApiResponse<LessonDetail>, ServerFnError> {
    let (response_options, db) = match get_server_context::<LessonDetail>().await {
        Ok(ctx) => ctx,
        Err(e) => {
            return Ok(ApiResponse {
                data: None,
                error: e.error,
            });
        }
    };
    let responder = ServerResponse::new(response_options);

    let lesson_id: RecordId = match parse_record_id(&lesson_id, "lesson_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let lesson: Option<Lesson> = db.select(lesson_id.clone()).await?;
    let lesson = match lesson {
        Some(lesson) if !lesson.deleted => lesson,
        None => return Ok(responder.not_found("Lesson not found".to_string())),
        _ => return Ok(responder.not_found("Lesson not found".to_string())),
    };

    let module: Option<Module> = db.select(lesson.module.clone()).await?;
    let module = match module {
        Some(module) if !module.deleted => module,
        None => return Ok(responder.not_found("Module not found".to_string())),
        _ => return Ok(responder.not_found("Module not found".to_string())),
    };

    let course: Option<Course> = db.select(module.course.clone()).await?;
    let course = match course {
        Some(course) if !course.deleted && course.status == CourseStatus::Published => course,
        _ => return Ok(responder.not_found("Course not found".to_string())),
    };

    let mut module_lessons_response = db
        .query(
            "SELECT * FROM lessons WHERE module = $module_id AND deleted = false ORDER BY sort_order ASC",
        )
        .bind(("module_id", module.id.clone()))
        .await?;
    let module_lessons: Vec<Lesson> = module_lessons_response.take(0)?;

    let mut next_lesson_id = None;
    let mut prev_lesson_id = None;
    for (idx, item) in module_lessons.iter().enumerate() {
        if item.id == lesson.id {
            if idx > 0 {
                prev_lesson_id = Some(module_lessons[idx - 1].id.to_string());
            }
            if idx + 1 < module_lessons.len() {
                next_lesson_id = Some(module_lessons[idx + 1].id.to_string());
            }
            break;
        }
    }

    let payload = LessonDetail {
        id: lesson.id.to_string(),
        title: lesson.title,
        content_type: lesson.content_type,
        content: lesson.content,
        video_url: lesson.video_url,
        video_duration_seconds: lesson.video_duration_seconds,
        audio_url: lesson.audio_url,
        pdf_url: lesson.pdf_url,
        external_url: lesson.external_url,
        thumbnail_url: lesson.thumbnail_url,
        duration_minutes: lesson.duration_minutes,
        module_id: module.id.to_string(),
        module_title: module.title,
        course_id: course.id.to_string(),
        course_title: course.title,
        is_completed: false,
        next_lesson_id,
        prev_lesson_id,
    };

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "search")]
pub async fn search_courses(
    keyword: String,
    level: Option<CourseLevel>,
) -> Result<ApiResponse<Vec<CourseOnClient>>, ServerFnError> {
    let (response_options, db) = match get_server_context::<Vec<CourseOnClient>>().await {
        Ok(ctx) => ctx,
        Err(e) => {
            return Ok(ApiResponse {
                data: None,
                error: e.error,
            });
        }
    };
    let responder = ServerResponse::new(response_options);

    let keyword = keyword.trim().to_lowercase();
    if keyword.is_empty() {
        return Ok(responder.ok(Vec::new()));
    }

    let mut response = db
        .query(
            "SELECT id, title, slug, description, short_description, level, thumbnail_url, video_url, duration_minutes, lesson_count, enrollment_count, educator FROM courses WHERE status = \"published\" AND deleted = false FETCH educator",
        )
        .await?;
    let courses: Vec<CourseWithEducator> = response.take(0)?;

    let filtered = courses
        .into_iter()
        .filter(|course| {
            let haystack = format!(
                "{} {} {} {}",
                course.title.to_lowercase(),
                course.description.to_lowercase(),
                course.short_description.to_lowercase(),
                course.slug.to_lowercase()
            );
            haystack.contains(&keyword) && level.as_ref().map_or(true, |lvl| lvl == &course.level)
        })
        .map(|course| CourseOnClient {
            id: course.id.to_string(),
            title: course.title,
            slug: course.slug,
            short_description: course.short_description,
            level: course.level,
            thumbnail_url: course.thumbnail_url,
            video_url: course.video_url,
            duration_minutes: course.duration_minutes,
            lesson_count: course.lesson_count,
            enrollment_count: course.enrollment_count,
            educator_name: course.educator.display_name,
        })
        .collect::<Vec<_>>();

    Ok(responder.ok(filtered))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "enroll")]
pub async fn enroll_course(course_id: String) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let course_id: RecordId = match parse_record_id(&course_id, "course_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let course: Option<Course> = db.select(course_id.clone()).await?;
    let course = match course {
        Some(course) => course,
        None => return Ok(responder.not_found("Course not found".to_string())),
    };

    if course.deleted || course.status != CourseStatus::Published {
        return Ok(responder.not_found("Course not found".to_string()));
    }

    let mut existing_response = db
        .query("SELECT id FROM enrolled WHERE in = $user_id AND out = $course_id LIMIT 1")
        .bind(("user_id", user.id.clone()))
        .bind(("course_id", course_id.clone()))
        .await?;
    let existing: Option<RelationId> = existing_response.take(0)?;
    if existing.is_some() {
        return Ok(responder.conflict("Already enrolled in course".to_string()));
    }

    let enroll_query = r#"
        BEGIN TRANSACTION;
        RELATE $user_id -> enrolled -> $course_id SET enrolled_at = time::now(), progress_percent = 0, last_accessed_at = time::now();
        COMMIT TRANSACTION;
    "#;
    let enroll_result = db
        .query(enroll_query)
        .bind(("user_id", user.id.clone()))
        .bind(("course_id", course_id.clone()))
        .await;

    if let Err(err) = enroll_result {
        error!(?err, "Failed to enroll in course");
        return Ok(responder.conflict("Already enrolled in course".to_string()));
    }

    let _ = update_course_enrollment_count(&course_id, &db).await;

    Ok(responder.ok("Enrolled successfully".to_string()))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "unenroll")]
pub async fn unenroll_course(course_id: String) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let course_id: RecordId = match parse_record_id(&course_id, "course_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let delete_query = "DELETE enrolled WHERE in = $user_id AND out = $course_id";
    db.query(delete_query)
        .bind(("user_id", user.id.clone()))
        .bind(("course_id", course_id.clone()))
        .await?;

    let _ = update_course_enrollment_count(&course_id, &db).await;

    Ok(responder.ok("Unenrolled successfully".to_string()))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "my-courses")]
pub async fn fetch_my_courses() -> Result<ApiResponse<Vec<EnrollmentProgress>>, ServerFnError> {
    let (response_options, db, user) =
        match get_authenticated_user_and_context::<Vec<EnrollmentProgress>>().await {
            Ok(ctx) => ctx,
            Err(e) => return Ok(e),
        };
    let responder = ServerResponse::new(response_options);

    let mut response = db
        .query("SELECT enrolled_at, progress_percent, last_accessed_at, out FROM enrolled WHERE in = $user_id FETCH out")
        .bind(("user_id", user.id.clone()))
        .await?;

    let rows: Vec<EnrolledWithCourse> = response.take(0)?;
    let payload = rows
        .into_iter()
        .map(|row| {
            let completed =
                ((row.progress_percent / 100.0) * row.out.lesson_count as f32).round() as i32;
            EnrollmentProgress {
                course_id: row.out.id.to_string(),
                course_title: row.out.title,
                thumbnail_url: row.out.thumbnail_url,
                enrolled_at: datetime_to_fixed(row.enrolled_at),
                progress_percent: row.progress_percent,
                completed_lessons: completed,
                total_lessons: row.out.lesson_count,
                last_accessed_at: row.last_accessed_at.map(datetime_to_fixed),
            }
        })
        .collect::<Vec<_>>();

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "complete-lesson")]
pub async fn complete_lesson(lesson_id: String) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let lesson_id: RecordId = match parse_record_id(&lesson_id, "lesson_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let lesson: Option<Lesson> = db.select(lesson_id.clone()).await?;
    let lesson = match lesson {
        Some(lesson) if !lesson.deleted => lesson,
        None => return Ok(responder.not_found("Lesson not found".to_string())),
        _ => return Ok(responder.not_found("Lesson not found".to_string())),
    };

    let module: Option<Module> = db.select(lesson.module.clone()).await?;
    let module = match module {
        Some(module) if !module.deleted => module,
        None => return Ok(responder.not_found("Module not found".to_string())),
        _ => return Ok(responder.not_found("Module not found".to_string())),
    };

    let course_id = module.course.clone();
    let mut enrolled_response = db
        .query("SELECT id FROM enrolled WHERE in = $user_id AND out = $course_id LIMIT 1")
        .bind(("user_id", user.id.clone()))
        .bind(("course_id", course_id.clone()))
        .await?;
    let enrolled: Option<RelationId> = enrolled_response.take(0)?;
    let enrolled = match enrolled {
        Some(enrolled) => enrolled,
        None => {
            return Ok(responder.forbidden("You are not enrolled in this course".to_string()));
        }
    };

    let mut completed_response = db
        .query("SELECT id FROM completed WHERE in = $user_id AND out = $lesson_id LIMIT 1")
        .bind(("user_id", user.id.clone()))
        .bind(("lesson_id", lesson_id.clone()))
        .await?;
    let completed: Option<RelationId> = completed_response.take(0)?;

    if completed.is_none() {
        let relate_query =
            "RELATE $user_id -> completed -> $lesson_id SET completed_at = time::now()";
        db.query(relate_query)
            .bind(("user_id", user.id.clone()))
            .bind(("lesson_id", lesson_id.clone()))
            .await?;
    }

    let mut course_lessons_response = db
        .query(
            "SELECT * FROM lessons WHERE module IN (SELECT VALUE id FROM modules WHERE course = $course_id AND deleted = false) AND deleted = false ORDER BY sort_order ASC",
        )
        .bind(("course_id", course_id.clone()))
        .await?;
    let course_lessons: Vec<Lesson> = course_lessons_response.take(0)?;
    let total_lessons = course_lessons.len() as f32;

    let lesson_ids = course_lessons
        .iter()
        .map(|lesson| lesson.id.to_string())
        .collect::<std::collections::HashSet<_>>();

    let mut completed_rows_response = db
        .query("SELECT out FROM completed WHERE in = $user_id")
        .bind(("user_id", user.id.clone()))
        .await?;
    let completed_rows: Vec<EnrolledWithLessonId> = completed_rows_response.take(0)?;
    let completed_lessons = completed_rows
        .into_iter()
        .filter(|row| lesson_ids.contains(&row.out.to_string()))
        .count() as f32;

    let progress_percent = if total_lessons > 0.0 {
        (completed_lessons / total_lessons) * 100.0
    } else {
        0.0
    };

    let last_accessed_at: Datetime = Utc::now().into();
    let completed_at: Option<Datetime> = if progress_percent >= 100.0 {
        Some(Utc::now().into())
    } else {
        None
    };
    db.query("UPDATE ONLY $enrollment_id MERGE $record")
        .bind(("enrollment_id", enrolled.id.clone()))
        .bind((
            "record",
            EnrollmentUpdate {
                progress_percent,
                last_accessed_at,
                completed_at,
            },
        ))
        .await?;

    if progress_percent >= 100.0 {
        let mut cert_response = db
            .query(
                "SELECT id FROM certificates WHERE user = $user_id AND course = $course_id LIMIT 1",
            )
            .bind(("user_id", user.id.clone()))
            .bind(("course_id", course_id.clone()))
            .await?;
        let existing_cert: Option<RelationId> = cert_response.take(0)?;
        if existing_cert.is_none() {
            let cert_number = generate_token();
            db.query("CREATE certificates CONTENT { user: $user_id, course: $course_id, certificate_number: $cert_number, issued_at: time::now() }")
                .bind(("user_id", user.id.clone()))
                .bind(("course_id", course_id.clone()))
                .bind(("cert_number", cert_number))
                .await?;
        }
    }

    let _ = bump_streak(&user.id, &db).await;
    let _ = check_and_award_achievements(&user.id, &db).await;

    Ok(responder.ok("Lesson marked as completed".to_string()))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "progress")]
pub async fn fetch_course_progress(
    course_id: String,
) -> Result<ApiResponse<EnrollmentProgress>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<EnrollmentProgress>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let course_id: RecordId = match parse_record_id(&course_id, "course_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let mut response = db
        .query("SELECT enrolled_at, progress_percent, last_accessed_at, out FROM enrolled WHERE in = $user_id AND out = $course_id FETCH out")
        .bind(("user_id", user.id.clone()))
        .bind(("course_id", course_id.clone()))
        .await?;
    let row: Option<EnrolledWithCourse> = response.take(0)?;

    let row = match row {
        Some(row) => row,
        None => return Ok(responder.not_found("Enrollment not found".to_string())),
    };

    let completed = ((row.progress_percent / 100.0) * row.out.lesson_count as f32).round() as i32;
    let payload = EnrollmentProgress {
        course_id: row.out.id.to_string(),
        course_title: row.out.title,
        thumbnail_url: row.out.thumbnail_url,
        enrolled_at: datetime_to_fixed(row.enrolled_at),
        progress_percent: row.progress_percent,
        completed_lessons: completed,
        total_lessons: row.out.lesson_count,
        last_accessed_at: row.last_accessed_at.map(datetime_to_fixed),
    };

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education/educator", endpoint = "courses")]
pub async fn fetch_educator_courses() -> Result<ApiResponse<Vec<CourseOnClient>>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<Vec<CourseOnClient>>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    if let Err(_) = is_educator_or_admin(&user) {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    let mut response = db
        .query("SELECT * FROM courses WHERE educator = $educator AND deleted = false")
        .bind(("educator", user.id.clone()))
        .await?;
    let courses: Vec<Course> = response.take(0)?;

    let payload = courses
        .into_iter()
        .map(|course| CourseOnClient {
            id: course.id.to_string(),
            title: course.title,
            slug: course.slug,
            short_description: course.short_description,
            level: course.level,
            thumbnail_url: course.thumbnail_url,
            video_url: course.video_url,
            duration_minutes: course.duration_minutes,
            lesson_count: course.lesson_count,
            enrollment_count: course.enrollment_count,
            educator_name: user.display_name.clone(),
        })
        .collect::<Vec<_>>();

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education/educator", endpoint = "courses-create")]
pub async fn create_course(
    create_course: CreateCourseInput,
) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    if let Err(_) = is_educator_or_admin(&user) {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    let validation_result = create_course.validate();
    if let Err(err) = validation_result {
        let errors = err
            .iter()
            .map(|(field, msg)| format!("{field}: {msg}"))
            .collect::<Vec<_>>();
        error!(?errors);
        return Ok(responder.unprocessable_entity("Invalid course data".to_string()));
    }

    let track: RecordId = match parse_record_id(&create_course.track, "track") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let now: Datetime = Utc::now().into();
    let record = CourseRecord {
        title: create_course.title,
        slug: create_course.slug,
        description: create_course.description,
        short_description: create_course.short_description,
        track,
        educator: user.id.clone(),
        level: create_course.level,
        status: CourseStatus::Draft,
        language: create_course.language,
        thumbnail_url: create_course.thumbnail_url,
        video_url: None,
        duration_minutes: 0,
        lesson_count: 0,
        enrollment_count: 0,
        created_at: now.clone(),
        updated_at: now,
        deleted: false,
    };

    let create_query = "CREATE ONLY courses CONTENT $course";
    let create_result = db.query(create_query).bind(("course", record)).await;
    if let Err(err) = create_result {
        error!(?err, "Failed to create course");
        return Ok(responder.internal_server_error("Failed to create course".to_string()));
    }

    Ok(responder.created("Course created".to_string()))
}

#[server(input = PatchJson, output = Json, prefix = "/education/educator", endpoint = "courses-update")]
pub async fn update_course(
    course_id: String,
    update: UpdateCourseInput,
) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let course_id: RecordId = match parse_record_id(&course_id, "course_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    if let Err(_) = is_educator_or_admin(&user) {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    if let Err(err) = update.validate() {
        let errors = err
            .iter()
            .map(|(field, msg)| format!("{field}: {msg}"))
            .collect::<Vec<_>>();
        error!(?errors);
        return Ok(responder.unprocessable_entity("Invalid course data".to_string()));
    }

    if let Err(_) = is_course_owner(&user, &course_id, &db).await {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    let track = match update.track {
        Some(track_id) => Some(match parse_record_id(&track_id, "track") {
            Ok(id) => id,
            Err(e) => return Ok(e),
        }),
        None => None,
    };

    let record = UpdatedCourseRecord {
        title: update.title,
        slug: update.slug,
        description: update.description,
        short_description: update.short_description,
        track,
        level: update.level,
        status: update.status,
        language: update.language,
        thumbnail_url: update.thumbnail_url,
        video_url: None,
        duration_minutes: update.duration_minutes,
        updated_at: Utc::now().into(),
    };

    let update_query = "UPDATE ONLY $course_id MERGE $record";
    let update_result = db
        .query(update_query)
        .bind(("course_id", course_id))
        .bind(("record", record))
        .await;

    if let Err(err) = update_result {
        error!(?err, "Failed to update course");
        return Ok(responder.internal_server_error("Failed to update course".to_string()));
    }

    Ok(responder.ok("Course updated".to_string()))
}

#[server(input = Json, output = Json, prefix = "/education/educator", endpoint = "courses-publish")]
pub async fn publish_course(course_id: String) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let course_id: RecordId = match parse_record_id(&course_id, "course_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    if let Err(_) = is_educator_or_admin(&user) {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    if let Err(_) = is_course_owner(&user, &course_id, &db).await {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    let mut module_count_response = db
        .query("SELECT count() AS count FROM modules WHERE course = $course_id AND deleted = false")
        .bind(("course_id", course_id.clone()))
        .await?;
    let module_counts: Vec<CountResult> = module_count_response.take(0)?;
    let module_count = module_counts.first().map(|c| c.count).unwrap_or(0);
    if module_count == 0 {
        return Ok(responder.bad_request("Course must have at least one module".to_string()));
    }

    let mut lesson_count_response = db
        .query(
            "SELECT count() AS count FROM lessons WHERE module IN (SELECT VALUE id FROM modules WHERE course = $course_id AND deleted = false) AND deleted = false",
        )
        .bind(("course_id", course_id.clone()))
        .await?;
    let lesson_counts: Vec<CountResult> = lesson_count_response.take(0)?;
    let lesson_count = lesson_counts.first().map(|c| c.count).unwrap_or(0);
    if lesson_count == 0 {
        return Ok(responder.bad_request("Course must have at least one lesson".to_string()));
    }

    let update_query =
        "UPDATE ONLY $course_id SET status = \"published\", updated_at = time::now()";
    db.query(update_query)
        .bind(("course_id", course_id))
        .await?;

    Ok(responder.ok("Course published".to_string()))
}

#[server(input = Json, output = Json, prefix = "/education/educator", endpoint = "modules-create")]
pub async fn create_module(
    create_module: CreateModuleInput,
) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    if let Err(_) = is_educator_or_admin(&user) {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    if let Err(err) = create_module.validate() {
        let errors = err
            .iter()
            .map(|(field, msg)| format!("{field}: {msg}"))
            .collect::<Vec<_>>();
        error!(?errors);
        return Ok(responder.unprocessable_entity("Invalid module data".to_string()));
    }

    let course_id: RecordId = match parse_record_id(&create_module.course, "course") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    if let Err(_) = is_course_owner(&user, &course_id, &db).await {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    let now: Datetime = Utc::now().into();
    let record = ModuleRecord {
        title: create_module.title,
        course: course_id.clone(),
        description: create_module.description,
        sort_order: create_module.sort_order.unwrap_or(0),
        created_at: now.clone(),
        updated_at: now,
        deleted: false,
    };

    let create_query = "CREATE ONLY modules CONTENT $module";
    db.query(create_query).bind(("module", record)).await?;

    Ok(responder.created("Module created".to_string()))
}

#[server(input = PatchJson, output = Json, prefix = "/education/educator", endpoint = "modules-update")]
pub async fn update_module(
    module_id: String,
    update: UpdateModuleInput,
) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    if let Err(_) = is_educator_or_admin(&user) {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    if let Err(err) = update.validate() {
        let errors = err
            .iter()
            .map(|(field, msg)| format!("{field}: {msg}"))
            .collect::<Vec<_>>();
        error!(?errors);
        return Ok(responder.unprocessable_entity("Invalid module data".to_string()));
    }

    let module_id: RecordId = match parse_record_id(&module_id, "module_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let module: Option<Module> = db.select(module_id.clone()).await?;
    let module = match module {
        Some(module) if !module.deleted => module,
        _ => return Ok(responder.not_found("Module not found".to_string())),
    };

    if let Err(_) = is_course_owner(&user, &module.course, &db).await {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    let record = UpdatedModuleRecord {
        title: update.title,
        description: update.description,
        sort_order: update.sort_order,
        updated_at: Utc::now().into(),
    };

    let update_query = "UPDATE ONLY $module_id MERGE $record";
    db.query(update_query)
        .bind(("module_id", module_id))
        .bind(("record", record))
        .await?;

    Ok(responder.ok("Module updated".to_string()))
}

#[server(input = DeleteUrl, output = Json, prefix = "/education/educator", endpoint = "modules-delete")]
pub async fn delete_module(module_id: String) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let module_id: RecordId = match parse_record_id(&module_id, "module_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let module: Option<Module> = db.select(module_id.clone()).await?;
    let module = match module {
        Some(module) if !module.deleted => module,
        _ => return Ok(responder.not_found("Module not found".to_string())),
    };

    if let Err(_) = is_course_owner(&user, &module.course, &db).await {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    let update_query = "UPDATE ONLY $module_id SET deleted = true, updated_at = time::now()";
    db.query(update_query)
        .bind(("module_id", module_id))
        .await?;

    db.query(
        "UPDATE lessons SET deleted = true, updated_at = time::now() WHERE module = $module_id",
    )
    .bind(("module_id", module.id.clone()))
    .await?;

    let _ = update_course_lesson_count(&module.course, &db).await;

    Ok(responder.ok("Module deleted".to_string()))
}

#[server(input = Json, output = Json, prefix = "/education/educator", endpoint = "lessons-create")]
pub async fn create_lesson(
    create_lesson: CreateLessonInput,
) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    if let Err(_) = is_educator_or_admin(&user) {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    if let Err(err) = create_lesson.validate() {
        let errors = err
            .iter()
            .map(|(field, msg)| format!("{field}: {msg}"))
            .collect::<Vec<_>>();
        error!(?errors);
        return Ok(responder.unprocessable_entity("Invalid lesson data".to_string()));
    }

    let module_id: RecordId = match parse_record_id(&create_lesson.module, "module") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let module: Option<Module> = db.select(module_id.clone()).await?;
    let module = match module {
        Some(module) if !module.deleted => module,
        _ => return Ok(responder.not_found("Module not found".to_string())),
    };

    if let Err(_) = is_course_owner(&user, &module.course, &db).await {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    let now: Datetime = Utc::now().into();
    let record = LessonRecord {
        title: create_lesson.title,
        module: module_id.clone(),
        content_type: create_lesson.content_type,
        content: create_lesson.content,
        video_url: create_lesson.video_url,
        video_duration_seconds: create_lesson.video_duration_seconds,
        audio_url: create_lesson.audio_url,
        pdf_url: create_lesson.pdf_url,
        external_url: create_lesson.external_url,
        thumbnail_url: create_lesson.thumbnail_url,
        duration_minutes: create_lesson.duration_minutes.unwrap_or(5),
        sort_order: create_lesson.sort_order.unwrap_or(0),
        is_preview: create_lesson.is_preview.unwrap_or(false),
        created_at: now.clone(),
        updated_at: now,
        deleted: false,
    };

    let create_query = "CREATE ONLY lessons CONTENT $lesson";
    db.query(create_query).bind(("lesson", record)).await?;

    let _ = update_course_lesson_count(&module.course, &db).await;

    Ok(responder.created("Lesson created".to_string()))
}

#[server(input = PatchJson, output = Json, prefix = "/education/educator", endpoint = "lessons-update")]
pub async fn update_lesson(
    lesson_id: String,
    update: UpdateLessonInput,
) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    if let Err(_) = is_educator_or_admin(&user) {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    if let Err(err) = update.validate() {
        let errors = err
            .iter()
            .map(|(field, msg)| format!("{field}: {msg}"))
            .collect::<Vec<_>>();
        error!(?errors);
        return Ok(responder.unprocessable_entity("Invalid lesson data".to_string()));
    }

    let lesson_id: RecordId = match parse_record_id(&lesson_id, "lesson_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let lesson: Option<Lesson> = db.select(lesson_id.clone()).await?;
    let lesson = match lesson {
        Some(lesson) if !lesson.deleted => lesson,
        _ => return Ok(responder.not_found("Lesson not found".to_string())),
    };

    let module: Option<Module> = db.select(lesson.module.clone()).await?;
    let module = match module {
        Some(module) if !module.deleted => module,
        _ => return Ok(responder.not_found("Module not found".to_string())),
    };

    if let Err(_) = is_course_owner(&user, &module.course, &db).await {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    let record = UpdatedLessonRecord {
        title: update.title,
        content_type: update.content_type,
        content: update.content,
        video_url: update.video_url,
        video_duration_seconds: update.video_duration_seconds,
        audio_url: update.audio_url,
        pdf_url: update.pdf_url,
        external_url: update.external_url,
        thumbnail_url: update.thumbnail_url,
        duration_minutes: update.duration_minutes,
        sort_order: update.sort_order,
        is_preview: update.is_preview,
        updated_at: Utc::now().into(),
    };

    let update_query = "UPDATE ONLY $lesson_id MERGE $record";
    db.query(update_query)
        .bind(("lesson_id", lesson_id))
        .bind(("record", record))
        .await?;

    let _ = update_course_lesson_count(&module.course, &db).await;

    Ok(responder.ok("Lesson updated".to_string()))
}

#[server(input = DeleteUrl, output = Json, prefix = "/education/educator", endpoint = "lessons-delete")]
pub async fn delete_lesson(lesson_id: String) -> Result<ApiResponse<String>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<String>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let lesson_id: RecordId = match parse_record_id(&lesson_id, "lesson_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let lesson: Option<Lesson> = db.select(lesson_id.clone()).await?;
    let lesson = match lesson {
        Some(lesson) if !lesson.deleted => lesson,
        _ => return Ok(responder.not_found("Lesson not found".to_string())),
    };

    let module: Option<Module> = db.select(lesson.module.clone()).await?;
    let module = match module {
        Some(module) if !module.deleted => module,
        _ => return Ok(responder.not_found("Module not found".to_string())),
    };

    if let Err(_) = is_course_owner(&user, &module.course, &db).await {
        return Ok(responder.unauthorized("Unauthorized".to_string()));
    }

    let update_query = "UPDATE ONLY $lesson_id SET deleted = true, updated_at = time::now()";
    db.query(update_query)
        .bind(("lesson_id", lesson_id))
        .await?;

    let _ = update_course_lesson_count(&module.course, &db).await;

    Ok(responder.ok("Lesson deleted".to_string()))
}
