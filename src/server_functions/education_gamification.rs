#[cfg(feature = "ssr")]
use chrono::{DateTime, FixedOffset, Utc};
use leptos::{prelude::ServerFnError, server_fn::codec::Json, *};
#[cfg(feature = "ssr")]
use serde::Deserialize;
#[cfg(feature = "ssr")]
use surrealdb::Datetime;

use crate::models::api_responses::ApiResponse;
#[cfg(feature = "ssr")]
use crate::models::gamification::{Achievement, Certificate};
use crate::models::gamification::{
    AchievementOnClient, CertificateOnClient, LeaderboardEntry, UserStreakOnClient,
};
#[cfg(feature = "ssr")]
use crate::models::user::User;
#[cfg(feature = "ssr")]
use crate::utils::ssr::{ServerResponse, get_authenticated_user_and_context};

#[cfg(feature = "ssr")]
fn datetime_to_fixed(datetime: Datetime) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(&datetime.to_string())
        .unwrap_or_else(|_| Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()))
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct EarnedWithAchievement {
    pub out: Achievement,
    pub earned_at: Datetime,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct UserStreakWithUser {
    pub user: User,
    pub current_streak: i32,
    pub longest_streak: i32,
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "streak")]
pub async fn fetch_streak() -> Result<ApiResponse<UserStreakOnClient>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<UserStreakOnClient>().await {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let streak: Option<UserStreakOnClient> = db
        .query("SELECT current_streak, longest_streak, last_activity_date FROM user_streaks WHERE user = $user_id LIMIT 1")
        .bind(("user_id", user.id))
        .await?
        .take(0)?;

    let payload = streak.unwrap_or(UserStreakOnClient {
        current_streak: 0,
        longest_streak: 0,
        last_activity_date: None,
    });

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "achievements")]
pub async fn fetch_achievements() -> Result<ApiResponse<Vec<AchievementOnClient>>, ServerFnError> {
    let (response_options, db, user) =
        match get_authenticated_user_and_context::<Vec<AchievementOnClient>>().await {
            Ok(ctx) => ctx,
            Err(e) => return Ok(e),
        };
    let responder = ServerResponse::new(response_options);

    let mut response = db
        .query("SELECT out, earned_at FROM earned WHERE in = $user_id FETCH out")
        .bind(("user_id", user.id))
        .await?;
    let rows: Vec<EarnedWithAchievement> = response.take(0)?;

    let payload = rows
        .into_iter()
        .map(|row| AchievementOnClient {
            id: row.out.id.to_string(),
            name: row.out.name,
            slug: row.out.slug,
            description: row.out.description,
            icon: row.out.icon,
            category: row.out.category,
            points: row.out.points,
            earned_at: Some(datetime_to_fixed(row.earned_at)),
        })
        .collect::<Vec<_>>();

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "certificates")]
pub async fn fetch_certificates() -> Result<ApiResponse<Vec<CertificateOnClient>>, ServerFnError> {
    let (response_options, db, user) =
        match get_authenticated_user_and_context::<Vec<CertificateOnClient>>().await {
            Ok(ctx) => ctx,
            Err(e) => return Ok(e),
        };
    let responder = ServerResponse::new(response_options);

    let mut response = db
        .query("SELECT * FROM certificates WHERE user = $user_id")
        .bind(("user_id", user.id))
        .await?;
    let certificates: Vec<Certificate> = response.take(0)?;

    let payload = certificates
        .into_iter()
        .map(|cert| CertificateOnClient {
            id: cert.id.to_string(),
            course_id: cert.course.to_string(),
            certificate_number: cert.certificate_number,
            issued_at: datetime_to_fixed(cert.issued_at),
            pdf_url: cert.pdf_url,
        })
        .collect::<Vec<_>>();

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "leaderboard")]
pub async fn fetch_leaderboard() -> Result<ApiResponse<Vec<LeaderboardEntry>>, ServerFnError> {
    let (response_options, db, _user) =
        match get_authenticated_user_and_context::<Vec<LeaderboardEntry>>().await {
            Ok(ctx) => ctx,
            Err(e) => return Ok(e),
        };
    let responder = ServerResponse::new(response_options);

    let mut response = db
        .query("SELECT user, current_streak, longest_streak FROM user_streaks FETCH user ORDER BY current_streak DESC LIMIT 20")
        .await?;
    let rows: Vec<UserStreakWithUser> = response.take(0)?;

    let payload = rows
        .into_iter()
        .map(|row| LeaderboardEntry {
            user_id: row.user.id.to_string(),
            display_name: row.user.display_name,
            current_streak: row.current_streak,
            longest_streak: row.longest_streak,
        })
        .collect::<Vec<_>>();

    Ok(responder.ok(payload))
}
