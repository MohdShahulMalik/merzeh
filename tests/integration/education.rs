use crate::common::get_test_db;
use merzah::{
    auth::session::create_session,
    models::{
        api_responses::ApiResponse,
        education::{
            Course, CourseDetail, CourseLevel, CourseOnClient, CourseRecord, CourseStatus,
            CreateCourse, CreateLesson, CreateModule, EnrollmentProgress, Lesson,
            LessonContentType, LessonDetail, LessonRecord, Module, ModuleRecord, Track,
            TrackOnClient, UpdateCourse, UpdateLesson, UpdateModule,
        },
        user::User,
    },
    spawn_app,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId};

#[derive(Serialize)]
struct EmptyBody {}

#[derive(Serialize)]
struct TrackIdParam {
    track_id: String,
}

#[derive(Serialize)]
struct CourseIdParam {
    course_id: String,
}

#[derive(Serialize)]
struct LessonIdParam {
    lesson_id: String,
}

#[derive(Serialize)]
struct SearchCoursesParams {
    keyword: String,
    level: Option<CourseLevel>,
}

#[derive(Serialize)]
struct CreateCourseParams {
    create_course: CreateCourse,
}

#[derive(Serialize)]
struct UpdateCourseParams {
    course_id: String,
    update: UpdateCourse,
}

#[derive(Serialize)]
struct CreateModuleParams {
    create_module: CreateModule,
}

#[derive(Serialize)]
struct UpdateModuleParams {
    module_id: String,
    update: UpdateModule,
}

#[derive(Serialize)]
struct CreateLessonParams {
    create_lesson: CreateLesson,
}

#[derive(Serialize)]
struct UpdateLessonParams {
    lesson_id: String,
    update: UpdateLesson,
}

#[derive(Serialize)]
struct TrackSeed {
    name: String,
    slug: String,
    description: String,
    icon: Option<String>,
    image_url: Option<String>,
    sort_order: i32,
    created_at: Datetime,
    updated_at: Datetime,
    deleted: bool,
}

#[derive(Serialize)]
struct AchievementSeed {
    name: String,
    slug: String,
    description: String,
    icon: String,
    category: String,
    requirement_type: String,
    requirement_value: i32,
    points: i32,
    created_at: Datetime,
}

#[derive(Serialize)]
struct CourseLessonCountUpdate {
    lesson_count: i32,
}

#[derive(Debug, Deserialize)]
struct CertificateRow {
    #[allow(dead_code)]
    id: RecordId,
}

#[derive(Debug, Deserialize)]
struct CompletedRow {
    #[allow(dead_code)]
    id: RecordId,
}

#[derive(Debug, Deserialize)]
struct StreakRow {
    current_streak: i32,
    longest_streak: i32,
}

fn auth_post(client: &Client, session: &str, url: &str) -> reqwest::RequestBuilder {
    client
        .post(url)
        .header("Authorization", format!("Bearer {}", session))
}

fn auth_patch(client: &Client, session: &str, url: &str) -> reqwest::RequestBuilder {
    client
        .patch(url)
        .header("Authorization", format!("Bearer {}", session))
}

fn auth_delete(client: &Client, session: &str, url: &str) -> reqwest::RequestBuilder {
    client
        .delete(url)
        .header("Authorization", format!("Bearer {}", session))
}

async fn load_education_schema(db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>) {
    let schema_files = [
        "schemas/tracks.surql",
        "schemas/courses.surql",
        "schemas/modules.surql",
        "schemas/lessons.surql",
        "schemas/enrolled.surql",
        "schemas/completed.surql",
        "schemas/certificates.surql",
        "schemas/achievements.surql",
        "schemas/earned.surql",
        "schemas/user_streaks.surql",
    ];

    for schema_path in schema_files {
        let mut content = tokio::fs::read_to_string(schema_path)
            .await
            .expect("failed to read education schema");

        content = content.replace(
            "DEFINE FIELD IF NOT EXISTS certificate_number ON certificates TYPE string UNIQUE;",
            "DEFINE FIELD IF NOT EXISTS certificate_number ON certificates TYPE string;",
        );
        content = content.replace(
            "DEFINE FIELD IF NOT EXISTS slug ON achievements TYPE string UNIQUE;",
            "DEFINE FIELD IF NOT EXISTS slug ON achievements TYPE string;",
        );
        content = content.replace(
            "DEFINE FIELD IF NOT EXISTS user ON user_streaks TYPE record<users> UNIQUE;",
            "DEFINE FIELD IF NOT EXISTS user ON user_streaks TYPE record<users>;",
        );

        db.query(content)
            .await
            .expect("failed to execute education schema");
    }

    db.query(
        r#"
        DEFINE INDEX IF NOT EXISTS certificate_number_idx ON certificates FIELDS certificate_number UNIQUE;
        DEFINE INDEX IF NOT EXISTS achievement_slug_idx ON achievements FIELDS slug UNIQUE;
        DEFINE INDEX IF NOT EXISTS user_streak_user_idx ON user_streaks FIELDS user UNIQUE;
        "#,
    )
    .await
    .expect("failed to execute education indexes");
}

async fn create_user_with_role(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    display_name: &str,
    role: &str,
) -> (User, String) {
    let user_id = RecordId::from(("users", format!("user_{}", uuid::Uuid::new_v4())));
    let user: User = db
        .create(user_id.clone())
        .content(User {
            id: user_id,
            created_at: Datetime::default(),
            display_name: display_name.to_string(),
            password_hash: "hash".to_string(),
            role: role.to_string(),
            updated_at: Datetime::default(),
        })
        .await
        .expect("failed to create user")
        .expect("user was not returned");

    let session = create_session(user.id.clone(), db)
        .await
        .expect("failed to create session");

    (user, session)
}

async fn create_track(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    name: &str,
    slug: &str,
    sort_order: i32,
) -> Track {
    db.create("tracks")
        .content(TrackSeed {
            name: name.to_string(),
            slug: slug.to_string(),
            description: format!("{name} description"),
            icon: Some("book-open".to_string()),
            image_url: None,
            sort_order,
            created_at: Datetime::default(),
            updated_at: Datetime::default(),
            deleted: false,
        })
        .await
        .expect("failed to create track")
        .expect("track was not returned")
}

async fn create_course_record(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    track_id: &RecordId,
    educator_id: &RecordId,
    title: &str,
    slug: &str,
    status: CourseStatus,
) -> Course {
    db.create("courses")
        .content(CourseRecord {
            title: title.to_string(),
            slug: slug.to_string(),
            description: format!("{title} full description for integration tests"),
            short_description: format!("{title} short intro"),
            track: track_id.clone(),
            educator: educator_id.clone(),
            level: CourseLevel::Beginner,
            status,
            language: "en".to_string(),
            thumbnail_url: Some("https://example.com/thumb.png".to_string()),
            video_url: None,
            duration_minutes: 0,
            lesson_count: 0,
            enrollment_count: 0,
            created_at: Datetime::default(),
            updated_at: Datetime::default(),
            deleted: false,
        })
        .await
        .expect("failed to create course")
        .expect("course was not returned")
}

async fn create_module_record(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    course_id: &RecordId,
    title: &str,
    description: Option<&str>,
    sort_order: i32,
) -> Module {
    db.create("modules")
        .content(ModuleRecord {
            title: title.to_string(),
            course: course_id.clone(),
            description: description.map(ToString::to_string),
            sort_order,
            created_at: Datetime::default(),
            updated_at: Datetime::default(),
            deleted: false,
        })
        .await
        .expect("failed to create module")
        .expect("module was not returned")
}

async fn create_lesson_record(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    module_id: &RecordId,
    title: &str,
    sort_order: i32,
    duration_minutes: i32,
) -> Lesson {
    db.create("lessons")
        .content(LessonRecord {
            title: title.to_string(),
            module: module_id.clone(),
            content_type: LessonContentType::Text,
            content: format!("{title} content"),
            video_url: None,
            video_duration_seconds: None,
            audio_url: None,
            pdf_url: None,
            external_url: None,
            thumbnail_url: None,
            duration_minutes,
            sort_order,
            is_preview: sort_order == 1,
            created_at: Datetime::default(),
            updated_at: Datetime::default(),
            deleted: false,
        })
        .await
        .expect("failed to create lesson")
        .expect("lesson was not returned")
}

async fn create_achievement(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    slug: &str,
    requirement_type: &str,
    requirement_value: i32,
) {
    let _: Option<RecordId> = db
        .query("CREATE achievements CONTENT $achievement RETURN VALUE id")
        .bind((
            "achievement",
            AchievementSeed {
                name: format!("Achievement {slug}"),
                slug: slug.to_string(),
                description: format!("Achievement for {slug}"),
                icon: "star".to_string(),
                category: "learning".to_string(),
                requirement_type: requirement_type.to_string(),
                requirement_value,
                points: 10,
                created_at: Datetime::default(),
            },
        ))
        .await
        .expect("failed to create achievement")
        .take(0)
        .expect("failed to parse achievement id");
}

#[tokio::test]
async fn education_public_endpoints_return_expected_data() {
    let db = get_test_db().await;
    load_education_schema(&db).await;

    let addr = spawn_app(db.clone());
    let client = Client::new();

    let (educator, _) = create_user_with_role(&db, "Teacher One", "educator").await;
    let track = create_track(&db, "Faith & Worship", "faith-worship", 1).await;
    let other_track = create_track(&db, "Life Skills", "life-skills", 2).await;

    let published_course = create_course_record(
        &db,
        &track.id,
        &educator.id,
        "Fiqh of Prayer",
        "fiqh-of-prayer",
        CourseStatus::Published,
    )
    .await;
    let _draft_course = create_course_record(
        &db,
        &track.id,
        &educator.id,
        "Draft Course",
        "draft-course",
        CourseStatus::Draft,
    )
    .await;
    let _other_track_course = create_course_record(
        &db,
        &other_track.id,
        &educator.id,
        "Time Management",
        "time-management",
        CourseStatus::Published,
    )
    .await;

    let module_basics = create_module_record(
        &db,
        &published_course.id,
        "Prayer Basics",
        Some("Core foundations"),
        1,
    )
    .await;
    let module_conditions = create_module_record(
        &db,
        &published_course.id,
        "Prayer Conditions",
        Some("Prerequisites"),
        2,
    )
    .await;

    let lesson_one = create_lesson_record(&db, &module_basics.id, "What Is Salah?", 1, 10).await;
    let lesson_two = create_lesson_record(&db, &module_basics.id, "Pillars of Salah", 2, 12).await;
    let _lesson_three =
        create_lesson_record(&db, &module_conditions.id, "Purification", 1, 8).await;

    let _: Option<Course> = db
        .update(published_course.id.clone())
        .merge(CourseLessonCountUpdate { lesson_count: 3 })
        .await
        .expect("failed to update course lesson count");

    let tracks_url = format!("{}/education/tracks", addr);
    let tracks_response = client
        .post(&tracks_url)
        .json(&EmptyBody {})
        .send()
        .await
        .expect("failed to fetch tracks");

    assert_eq!(tracks_response.status().as_u16(), 200);

    let tracks_body: ApiResponse<Vec<TrackOnClient>> = tracks_response
        .json()
        .await
        .expect("failed to deserialize tracks response");

    let tracks = tracks_body.data.expect("tracks payload missing");
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0].slug, "faith-worship");
    assert_eq!(tracks[0].course_count, 1);
    assert_eq!(tracks[1].slug, "life-skills");
    assert_eq!(tracks[1].course_count, 1);

    let track_courses_url = format!("{}/education/track-courses", addr);
    let track_courses_response = client
        .post(&track_courses_url)
        .json(&TrackIdParam {
            track_id: track.id.to_string(),
        })
        .send()
        .await
        .expect("failed to fetch track courses");

    assert_eq!(track_courses_response.status().as_u16(), 200);

    let track_courses_body: ApiResponse<Vec<CourseOnClient>> = track_courses_response
        .json()
        .await
        .expect("failed to deserialize track courses response");

    let courses = track_courses_body.data.expect("course payload missing");
    assert_eq!(courses.len(), 1);
    assert_eq!(courses[0].title, "Fiqh of Prayer");
    assert_eq!(courses[0].educator_name, "Teacher One");

    let course_url = format!("{}/education/course", addr);
    let course_response = client
        .post(&course_url)
        .json(&CourseIdParam {
            course_id: published_course.id.to_string(),
        })
        .send()
        .await
        .expect("failed to fetch course details");

    assert_eq!(course_response.status().as_u16(), 200);

    let course_body: ApiResponse<CourseDetail> = course_response
        .json()
        .await
        .expect("failed to deserialize course details");

    let course_detail = course_body.data.expect("course detail missing");
    assert_eq!(course_detail.title, "Fiqh of Prayer");
    assert_eq!(course_detail.modules.len(), 2);
    assert_eq!(course_detail.modules[0].title, "Prayer Basics");
    assert_eq!(course_detail.modules[0].lessons.len(), 2);
    assert_eq!(course_detail.modules[0].lessons[0].title, "What Is Salah?");
    assert_eq!(course_detail.modules[1].title, "Prayer Conditions");

    let lesson_url = format!("{}/education/lesson", addr);
    let lesson_response = client
        .post(&lesson_url)
        .json(&LessonIdParam {
            lesson_id: lesson_two.id.to_string(),
        })
        .send()
        .await
        .expect("failed to fetch lesson details");

    assert_eq!(lesson_response.status().as_u16(), 200);

    let lesson_body: ApiResponse<LessonDetail> = lesson_response
        .json()
        .await
        .expect("failed to deserialize lesson details");

    let lesson_detail = lesson_body.data.expect("lesson detail missing");
    assert_eq!(lesson_detail.title, "Pillars of Salah");
    assert_eq!(
        lesson_detail.prev_lesson_id,
        Some(lesson_one.id.to_string())
    );
    assert_eq!(lesson_detail.next_lesson_id, None);
    assert_eq!(lesson_detail.course_title, "Fiqh of Prayer");

    let search_url = format!("{}/education/search", addr);
    let search_response = client
        .post(&search_url)
        .json(&SearchCoursesParams {
            keyword: "prayer".to_string(),
            level: Some(CourseLevel::Beginner),
        })
        .send()
        .await
        .expect("failed to search courses");

    assert_eq!(search_response.status().as_u16(), 200);

    let search_body: ApiResponse<Vec<CourseOnClient>> = search_response
        .json()
        .await
        .expect("failed to deserialize search response");

    let matches = search_body.data.expect("search payload missing");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].slug, "fiqh-of-prayer");

    let empty_search_response = client
        .post(&search_url)
        .json(&SearchCoursesParams {
            keyword: "   ".to_string(),
            level: None,
        })
        .send()
        .await
        .expect("failed to search courses with empty keyword");

    assert_eq!(empty_search_response.status().as_u16(), 200);

    let empty_search_body: ApiResponse<Vec<CourseOnClient>> = empty_search_response
        .json()
        .await
        .expect("failed to deserialize empty search response");

    assert!(
        empty_search_body
            .data
            .expect("empty search payload missing")
            .is_empty()
    );
}

#[tokio::test]
async fn education_enrollment_flow_updates_progress_and_certificate() {
    let db = get_test_db().await;
    load_education_schema(&db).await;

    let addr = spawn_app(db.clone());
    let client = Client::new();

    create_achievement(&db, "first-lesson", "lessons_completed", 1).await;

    let (educator, _) = create_user_with_role(&db, "Teacher Two", "educator").await;
    let (student, student_session) = create_user_with_role(&db, "Student One", "regular").await;
    let track = create_track(&db, "Quran", "quran", 1).await;
    let course = create_course_record(
        &db,
        &track.id,
        &educator.id,
        "Tajweed Basics",
        "tajweed-basics",
        CourseStatus::Published,
    )
    .await;
    let module = create_module_record(&db, &course.id, "Letters", Some("Makharij"), 1).await;
    let lesson_one = create_lesson_record(&db, &module.id, "Letter Points", 1, 7).await;
    let lesson_two = create_lesson_record(&db, &module.id, "Heavy Letters", 2, 9).await;

    let _: Option<Course> = db
        .update(course.id.clone())
        .merge(CourseLessonCountUpdate { lesson_count: 2 })
        .await
        .expect("failed to update course lesson count");

    let complete_url = format!("{}/education/complete-lesson", addr);
    let forbidden_response = auth_post(&client, &student_session, &complete_url)
        .json(&LessonIdParam {
            lesson_id: lesson_one.id.to_string(),
        })
        .send()
        .await
        .expect("failed to call complete lesson before enrolling");

    assert_eq!(forbidden_response.status().as_u16(), 403);

    let enroll_url = format!("{}/education/enroll", addr);
    let enroll_response = auth_post(&client, &student_session, &enroll_url)
        .json(&CourseIdParam {
            course_id: course.id.to_string(),
        })
        .send()
        .await
        .expect("failed to enroll in course");

    assert_eq!(enroll_response.status().as_u16(), 200);

    let enroll_body: ApiResponse<String> = enroll_response
        .json()
        .await
        .expect("failed to deserialize enroll response");

    assert_eq!(enroll_body.data, Some("Enrolled successfully".to_string()));

    let duplicate_response = auth_post(&client, &student_session, &enroll_url)
        .json(&CourseIdParam {
            course_id: course.id.to_string(),
        })
        .send()
        .await
        .expect("failed to call duplicate enroll");

    assert_eq!(duplicate_response.status().as_u16(), 409);

    let duplicate_body: ApiResponse<String> = duplicate_response
        .json()
        .await
        .expect("failed to deserialize duplicate enroll response");
    assert_eq!(
        duplicate_body.error,
        Some("Already enrolled in course".to_string())
    );

    let my_courses_url = format!("{}/education/my-courses", addr);
    let my_courses_response = auth_post(&client, &student_session, &my_courses_url)
        .json(&EmptyBody {})
        .send()
        .await
        .expect("failed to fetch my courses");

    assert_eq!(my_courses_response.status().as_u16(), 200);

    let my_courses_body: ApiResponse<Vec<EnrollmentProgress>> = my_courses_response
        .json()
        .await
        .expect("failed to deserialize my courses response");

    let my_courses = my_courses_body.data.expect("my courses payload missing");
    assert_eq!(my_courses.len(), 1);
    assert_eq!(my_courses[0].course_title, "Tajweed Basics");
    assert_eq!(my_courses[0].progress_percent, 0.0);

    let first_complete_response = auth_post(&client, &student_session, &complete_url)
        .json(&LessonIdParam {
            lesson_id: lesson_one.id.to_string(),
        })
        .send()
        .await
        .expect("failed to complete first lesson");

    assert_eq!(first_complete_response.status().as_u16(), 200);

    let progress_url = format!("{}/education/progress", addr);
    let first_progress_response = auth_post(&client, &student_session, &progress_url)
        .json(&CourseIdParam {
            course_id: course.id.to_string(),
        })
        .send()
        .await
        .expect("failed to fetch first progress");

    assert_eq!(first_progress_response.status().as_u16(), 200);

    let first_progress_body: ApiResponse<EnrollmentProgress> = first_progress_response
        .json()
        .await
        .expect("failed to deserialize first progress response");

    let first_progress = first_progress_body
        .data
        .expect("first progress payload missing");
    assert_eq!(first_progress.progress_percent, 50.0);
    assert_eq!(first_progress.completed_lessons, 1);
    assert_eq!(first_progress.total_lessons, 2);

    let second_complete_response = auth_post(&client, &student_session, &complete_url)
        .json(&LessonIdParam {
            lesson_id: lesson_two.id.to_string(),
        })
        .send()
        .await
        .expect("failed to complete second lesson");

    assert_eq!(second_complete_response.status().as_u16(), 200);

    let final_progress_response = auth_post(&client, &student_session, &progress_url)
        .json(&CourseIdParam {
            course_id: course.id.to_string(),
        })
        .send()
        .await
        .expect("failed to fetch final progress");

    assert_eq!(final_progress_response.status().as_u16(), 200);

    let final_progress_body: ApiResponse<EnrollmentProgress> = final_progress_response
        .json()
        .await
        .expect("failed to deserialize final progress response");

    let final_progress = final_progress_body
        .data
        .expect("final progress payload missing");
    assert_eq!(final_progress.progress_percent, 100.0);
    assert_eq!(final_progress.completed_lessons, 2);

    let completed_rows: Vec<CompletedRow> = db
        .query("SELECT id FROM completed WHERE in = $user_id")
        .bind(("user_id", student.id.clone()))
        .await
        .expect("failed to query completed lessons")
        .take(0)
        .expect("failed to parse completed lessons");
    assert_eq!(completed_rows.len(), 2);

    let certificate_rows: Vec<CertificateRow> = db
        .query("SELECT id FROM certificates WHERE user = $user_id AND course = $course_id")
        .bind(("user_id", student.id.clone()))
        .bind(("course_id", course.id.clone()))
        .await
        .expect("failed to query certificates")
        .take(0)
        .expect("failed to parse certificates");
    assert_eq!(certificate_rows.len(), 1);

    let streak_row: Option<StreakRow> = db
        .query(
            "SELECT current_streak, longest_streak FROM user_streaks WHERE user = $user_id LIMIT 1",
        )
        .bind(("user_id", student.id.clone()))
        .await
        .expect("failed to query streak")
        .take(0)
        .expect("failed to parse streak row");
    let streak_row = streak_row.expect("streak row missing");
    assert_eq!(streak_row.current_streak, 1);
    assert_eq!(streak_row.longest_streak, 1);

    let course_after_enroll: Option<Course> = db
        .select(course.id.clone())
        .await
        .expect("failed to reselect course");
    assert_eq!(
        course_after_enroll
            .expect("course missing")
            .enrollment_count,
        1
    );

    let unenroll_url = format!("{}/education/unenroll", addr);
    let unenroll_response = auth_post(&client, &student_session, &unenroll_url)
        .json(&CourseIdParam {
            course_id: course.id.to_string(),
        })
        .send()
        .await
        .expect("failed to unenroll from course");

    assert_eq!(unenroll_response.status().as_u16(), 200);

    let my_courses_after_response = auth_post(&client, &student_session, &my_courses_url)
        .json(&EmptyBody {})
        .send()
        .await
        .expect("failed to fetch my courses after unenroll");

    let my_courses_after_body: ApiResponse<Vec<EnrollmentProgress>> = my_courses_after_response
        .json()
        .await
        .expect("failed to deserialize my courses after unenroll");

    assert!(
        my_courses_after_body
            .data
            .expect("my courses after unenroll payload missing")
            .is_empty()
    );
}

#[tokio::test]
async fn education_educator_endpoints_manage_course_module_and_lesson() {
    let db = get_test_db().await;
    load_education_schema(&db).await;

    let addr = spawn_app(db.clone());
    let client = Client::new();

    let (educator, session) = create_user_with_role(&db, "Educator Owner", "educator").await;
    let track = create_track(&db, "Career", "career", 1).await;

    let educator_courses_url = format!("{}/education/educator/courses", addr);
    let empty_courses_response = auth_post(&client, &session, &educator_courses_url)
        .json(&EmptyBody {})
        .send()
        .await
        .expect("failed to fetch educator courses");

    assert_eq!(empty_courses_response.status().as_u16(), 200);

    let empty_courses_body: ApiResponse<Vec<CourseOnClient>> = empty_courses_response
        .json()
        .await
        .expect("failed to deserialize empty educator courses response");
    assert!(
        empty_courses_body
            .data
            .expect("educator courses payload missing")
            .is_empty()
    );

    let create_course_url = format!("{}/education/educator/courses-create", addr);
    let create_course_response = auth_post(&client, &session, &create_course_url)
        .json(&CreateCourseParams {
            create_course: CreateCourse {
                title: "Career Planning".to_string(),
                slug: "career-planning".to_string(),
                description: "A practical guide to ethical career planning.".to_string(),
                short_description: "Build a career plan with ihsan.".to_string(),
                track: track.id.to_string(),
                level: CourseLevel::Intermediate,
                language: "en".to_string(),
                thumbnail_url: Some("https://example.com/career.png".to_string()),
                video_url: None,
            },
        })
        .send()
        .await
        .expect("failed to create course");

    assert_eq!(create_course_response.status().as_u16(), 201);

    let created_course: Option<Course> = db
        .query("SELECT * FROM courses WHERE slug = $slug LIMIT 1")
        .bind(("slug", "career-planning"))
        .await
        .expect("failed to query created course")
        .take(0)
        .expect("failed to parse created course");
    let created_course = created_course.expect("created course missing");
    assert_eq!(created_course.educator, educator.id);
    assert_eq!(created_course.status, CourseStatus::Draft);

    let created_courses_response = auth_post(&client, &session, &educator_courses_url)
        .json(&EmptyBody {})
        .send()
        .await
        .expect("failed to fetch educator courses after create");

    let created_courses_body: ApiResponse<Vec<CourseOnClient>> = created_courses_response
        .json()
        .await
        .expect("failed to deserialize educator courses after create");
    let created_courses = created_courses_body
        .data
        .expect("educator courses after create payload missing");
    assert_eq!(created_courses.len(), 1);
    assert_eq!(created_courses[0].title, "Career Planning");

    let publish_course_url = format!("{}/education/educator/courses-publish", addr);
    let publish_without_content_response = auth_post(&client, &session, &publish_course_url)
        .json(&CourseIdParam {
            course_id: created_course.id.to_string(),
        })
        .send()
        .await
        .expect("failed to publish empty course");

    assert_eq!(publish_without_content_response.status().as_u16(), 400);

    let update_course_url = format!("{}/education/educator/courses-update", addr);
    let update_course_response = auth_patch(&client, &session, &update_course_url)
        .json(&UpdateCourseParams {
            course_id: created_course.id.to_string(),
            update: UpdateCourse {
                title: Some("Career Planning Essentials".to_string()),
                slug: None,
                description: None,
                short_description: None,
                track: None,
                level: Some(CourseLevel::Advanced),
                status: None,
                language: Some("en-US".to_string()),
                thumbnail_url: Some("https://example.com/career-new.png".to_string()),
                video_url: None,
                duration_minutes: Some(45),
            },
        })
        .send()
        .await
        .expect("failed to update course");

    assert_eq!(update_course_response.status().as_u16(), 200);

    let updated_course: Option<Course> = db
        .select(created_course.id.clone())
        .await
        .expect("failed to select updated course");
    let updated_course = updated_course.expect("updated course missing");
    assert_eq!(updated_course.title, "Career Planning Essentials");
    assert_eq!(updated_course.level, CourseLevel::Advanced);
    assert_eq!(updated_course.duration_minutes, 45);

    let create_module_url = format!("{}/education/educator/modules-create", addr);
    let create_module_response = auth_post(&client, &session, &create_module_url)
        .json(&CreateModuleParams {
            create_module: CreateModule {
                title: "Career Foundations".to_string(),
                course: created_course.id.to_string(),
                description: Some("Foundational concepts".to_string()),
                sort_order: Some(3),
            },
        })
        .send()
        .await
        .expect("failed to create module");

    assert_eq!(create_module_response.status().as_u16(), 201);

    let module: Option<Module> = db
        .query("SELECT * FROM modules WHERE course = $course_id LIMIT 1")
        .bind(("course_id", created_course.id.clone()))
        .await
        .expect("failed to query created module")
        .take(0)
        .expect("failed to parse created module");
    let module = module.expect("created module missing");
    assert_eq!(module.title, "Career Foundations");
    assert_eq!(module.sort_order, 3);

    let update_module_url = format!("{}/education/educator/modules-update", addr);
    let update_module_response = auth_patch(&client, &session, &update_module_url)
        .json(&UpdateModuleParams {
            module_id: module.id.to_string(),
            update: UpdateModule {
                title: Some("Career Foundations Updated".to_string()),
                description: Some("Updated module description".to_string()),
                sort_order: Some(1),
            },
        })
        .send()
        .await
        .expect("failed to update module");

    assert_eq!(update_module_response.status().as_u16(), 200);

    let updated_module: Option<Module> = db
        .select(module.id.clone())
        .await
        .expect("failed to select updated module");
    let updated_module = updated_module.expect("updated module missing");
    assert_eq!(updated_module.title, "Career Foundations Updated");
    assert_eq!(updated_module.sort_order, 1);

    let create_lesson_url = format!("{}/education/educator/lessons-create", addr);
    let create_lesson_response = auth_post(&client, &session, &create_lesson_url)
        .json(&CreateLessonParams {
            create_lesson: CreateLesson {
                title: "Set Intentional Goals".to_string(),
                module: module.id.to_string(),
                content_type: LessonContentType::Text,
                content: "Goal-setting content".to_string(),
                video_url: None,
                video_duration_seconds: None,
                audio_url: None,
                pdf_url: None,
                external_url: None,
                thumbnail_url: None,
                duration_minutes: Some(14),
                sort_order: Some(2),
                is_preview: Some(true),
            },
        })
        .send()
        .await
        .expect("failed to create lesson");

    assert_eq!(create_lesson_response.status().as_u16(), 201);

    let lesson: Option<Lesson> = db
        .query("SELECT * FROM lessons WHERE module = $module_id LIMIT 1")
        .bind(("module_id", module.id.clone()))
        .await
        .expect("failed to query created lesson")
        .take(0)
        .expect("failed to parse created lesson");
    let lesson = lesson.expect("created lesson missing");
    assert_eq!(lesson.title, "Set Intentional Goals");
    assert_eq!(lesson.duration_minutes, 14);
    assert!(lesson.is_preview);

    let course_after_lesson: Option<Course> = db
        .select(created_course.id.clone())
        .await
        .expect("failed to select course after lesson create");
    assert_eq!(
        course_after_lesson
            .expect("course after lesson create missing")
            .lesson_count,
        1
    );

    let update_lesson_url = format!("{}/education/educator/lessons-update", addr);
    let update_lesson_response = auth_patch(&client, &session, &update_lesson_url)
        .json(&UpdateLessonParams {
            lesson_id: lesson.id.to_string(),
            update: UpdateLesson {
                title: Some("Set Clear Goals".to_string()),
                content_type: Some(LessonContentType::Video),
                content: Some("Updated goal-setting content".to_string()),
                video_url: Some("https://example.com/video.mp4".to_string()),
                video_duration_seconds: Some(600),
                audio_url: None,
                pdf_url: None,
                external_url: None,
                thumbnail_url: Some("https://example.com/lesson.png".to_string()),
                duration_minutes: Some(10),
                sort_order: Some(1),
                is_preview: Some(false),
            },
        })
        .send()
        .await
        .expect("failed to update lesson");

    assert_eq!(update_lesson_response.status().as_u16(), 200);

    let updated_lesson: Option<Lesson> = db
        .select(lesson.id.clone())
        .await
        .expect("failed to select updated lesson");
    let updated_lesson = updated_lesson.expect("updated lesson missing");
    assert_eq!(updated_lesson.title, "Set Clear Goals");
    assert_eq!(updated_lesson.content_type, LessonContentType::Video);
    assert_eq!(updated_lesson.sort_order, 1);
    assert!(!updated_lesson.is_preview);

    let publish_course_response = auth_post(&client, &session, &publish_course_url)
        .json(&CourseIdParam {
            course_id: created_course.id.to_string(),
        })
        .send()
        .await
        .expect("failed to publish course");

    assert_eq!(publish_course_response.status().as_u16(), 200);

    let published_course: Option<Course> = db
        .select(created_course.id.clone())
        .await
        .expect("failed to select published course");
    assert_eq!(
        published_course.expect("published course missing").status,
        CourseStatus::Published
    );

    let delete_lesson_url = format!("{}/education/educator/lessons-delete", addr);
    let delete_lesson_response = auth_delete(&client, &session, &delete_lesson_url)
        .query(&[("lesson_id", lesson.id.to_string())])
        .send()
        .await
        .expect("failed to delete lesson");

    assert_eq!(delete_lesson_response.status().as_u16(), 200);

    let deleted_lesson: Option<Lesson> = db
        .select(lesson.id.clone())
        .await
        .expect("failed to select deleted lesson");
    assert!(deleted_lesson.expect("deleted lesson missing").deleted);

    let delete_module_url = format!("{}/education/educator/modules-delete", addr);
    let delete_module_response = auth_delete(&client, &session, &delete_module_url)
        .query(&[("module_id", module.id.to_string())])
        .send()
        .await
        .expect("failed to delete module");

    assert_eq!(delete_module_response.status().as_u16(), 200);

    let deleted_module: Option<Module> = db
        .select(module.id.clone())
        .await
        .expect("failed to select deleted module");
    assert!(deleted_module.expect("deleted module missing").deleted);

    let course_after_module_delete: Option<Course> = db
        .select(created_course.id.clone())
        .await
        .expect("failed to select course after module delete");
    assert_eq!(
        course_after_module_delete
            .expect("course after module delete missing")
            .lesson_count,
        0
    );
}
