use leptos::{prelude::ServerFnError, server_fn::codec::Json, *};
#[cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::RecordId;

use crate::models::api_responses::ApiResponse;
#[cfg(feature = "ssr")]
use crate::models::quiz::{Quiz, QuizQuestion};
use crate::models::quiz::{
    QuizAnswer, QuizOnClient, QuizQuestionOnClient, QuizSubmission, QuizSubmissionResult,
};
#[cfg(feature = "ssr")]
use crate::utils::parsing::parse_record_id;
#[cfg(feature = "ssr")]
use crate::utils::ssr::{ServerResponse, get_authenticated_user_and_context, get_server_context};

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct CountResult {
    pub count: i64,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Serialize)]
struct QuizAttemptRecord {
    pub user: RecordId,
    pub quiz: RecordId,
    pub answers: Vec<QuizAnswer>,
    pub score: f32,
    pub passed: bool,
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "quiz")]
pub async fn fetch_quiz_for_lesson(
    lesson_id: String,
) -> Result<ApiResponse<QuizOnClient>, ServerFnError> {
    let (response_options, db) = match get_server_context::<QuizOnClient>().await {
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

    let mut quiz_response = db
        .query("SELECT * FROM quizzes WHERE lesson = $lesson_id LIMIT 1")
        .bind(("lesson_id", lesson_id.clone()))
        .await?;
    let quiz: Option<Quiz> = quiz_response.take(0)?;
    let quiz = match quiz {
        Some(quiz) => quiz,
        None => return Ok(responder.not_found("Quiz not found".to_string())),
    };

    let mut question_response = db
        .query("SELECT * FROM quiz_questions WHERE quiz = $quiz_id ORDER BY sort_order ASC")
        .bind(("quiz_id", quiz.id.clone()))
        .await?;
    let questions: Vec<QuizQuestion> = question_response.take(0)?;

    let payload = QuizOnClient {
        id: quiz.id.to_string(),
        lesson_id: quiz.lesson.to_string(),
        title: quiz.title,
        passing_score: quiz.passing_score,
        max_attempts: quiz.max_attempts,
        questions: questions
            .into_iter()
            .map(|question| QuizQuestionOnClient {
                id: question.id.to_string(),
                question_text: question.question_text,
                question_type: question.question_type,
                options: question.options,
                sort_order: question.sort_order,
            })
            .collect(),
    };

    Ok(responder.ok(payload))
}

#[server(input = Json, output = Json, prefix = "/education", endpoint = "quiz-submit")]
pub async fn submit_quiz(
    submission: QuizSubmission,
) -> Result<ApiResponse<QuizSubmissionResult>, ServerFnError> {
    let (response_options, db, user) = match get_authenticated_user_and_context::<QuizSubmissionResult>().await
    {
        Ok(ctx) => ctx,
        Err(e) => return Ok(e),
    };
    let responder = ServerResponse::new(response_options);

    let quiz_id: RecordId = match parse_record_id(&submission.quiz_id, "quiz_id") {
        Ok(id) => id,
        Err(e) => return Ok(e),
    };

    let mut quiz_response = db
        .query("SELECT * FROM quizzes WHERE id = $quiz_id LIMIT 1")
        .bind(("quiz_id", quiz_id.clone()))
        .await?;
    let quiz: Option<Quiz> = quiz_response.take(0)?;
    let quiz = match quiz {
        Some(quiz) => quiz,
        None => return Ok(responder.not_found("Quiz not found".to_string())),
    };

    let mut attempt_count_response = db
        .query(
            "SELECT count() AS count FROM quiz_attempts WHERE user = $user_id AND quiz = $quiz_id",
        )
        .bind(("user_id", user.id.clone()))
        .bind(("quiz_id", quiz_id.clone()))
        .await?;
    let counts: Vec<CountResult> = attempt_count_response.take(0)?;
    let attempts = counts.first().map(|c| c.count).unwrap_or(0);
    if attempts >= quiz.max_attempts as i64 {
        return Ok(responder.bad_request("Max attempts reached".to_string()));
    }

    let mut question_response = db
        .query("SELECT * FROM quiz_questions WHERE quiz = $quiz_id ORDER BY sort_order ASC")
        .bind(("quiz_id", quiz_id.clone()))
        .await?;
    let questions: Vec<QuizQuestion> = question_response.take(0)?;

    let mut correct_count = 0;
    for question in &questions {
        if let Some(answer) = submission
            .answers
            .iter()
            .find(|a| a.question_id == question.id.to_string())
        {
            let expected = question.correct_answer.trim().to_lowercase();
            let actual = answer.answer.trim().to_lowercase();
            if expected == actual {
                correct_count += 1;
            }
        }
    }

    let total_questions = questions.len() as i32;
    let score = if total_questions > 0 {
        correct_count as f32 / total_questions as f32
    } else {
        0.0
    };
    let passed = score >= quiz.passing_score;

    let attempt_record = QuizAttemptRecord {
        user: user.id.clone(),
        quiz: quiz.id.clone(),
        answers: submission.answers.clone(),
        score,
        passed,
    };

    db.query("CREATE quiz_attempts CONTENT $attempt")
        .bind(("attempt", attempt_record))
        .await?;

    let payload = QuizSubmissionResult {
        score,
        passed,
        correct_count,
        total_questions,
    };

    Ok(responder.ok(payload))
}
