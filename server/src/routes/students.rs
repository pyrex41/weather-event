use crate::{error::ApiResult, AppState};
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use core::models::{Student, TrainingLevel};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateStudentRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 1, message = "Phone cannot be empty"))]
    pub phone: String,
    pub training_level: String,
}

#[derive(Debug, Serialize)]
pub struct StudentResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub training_level: String,
}

impl From<Student> for StudentResponse {
    fn from(student: Student) -> Self {
        Self {
            id: student.id,
            name: student.name,
            email: student.email,
            phone: student.phone,
            training_level: student.training_level.as_str().to_string(),
        }
    }
}

pub async fn list_students(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<StudentResponse>>> {
    let students = sqlx::query_as::<_, Student>(
        "SELECT id, name, email, phone, training_level FROM students ORDER BY name"
    )
    .fetch_all(&state.db)
    .await?;

    tracing::debug!("Retrieved {} students", students.len());
    Ok(Json(students.into_iter().map(StudentResponse::from).collect()))
}

pub async fn create_student(
    State(state): State<AppState>,
    Json(req): Json<CreateStudentRequest>,
) -> ApiResult<(StatusCode, Json<StudentResponse>)> {
    // Validate input fields
    req.validate()
        .map_err(|e| crate::error::ApiError::validation_error(e.to_string()))?;

    // Validate training level
    let training_level = match req.training_level.as_str() {
        "STUDENT_PILOT" => TrainingLevel::StudentPilot,
        "PRIVATE_PILOT" => TrainingLevel::PrivatePilot,
        "INSTRUMENT_RATED" => TrainingLevel::InstrumentRated,
        _ => {
            return Err(crate::error::ApiError::validation_error(
                format!("Invalid training level: {}. Must be one of: STUDENT_PILOT, PRIVATE_PILOT, INSTRUMENT_RATED", req.training_level)
            ));
        }
    };

    // Generate UUID
    let id = uuid::Uuid::new_v4().to_string();

    // Insert student
    sqlx::query(
        "INSERT INTO students (id, name, email, phone, training_level) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(training_level.as_str())
    .execute(&state.db)
    .await?;

    // Fetch created student
    let student = sqlx::query_as::<_, Student>(
        "SELECT id, name, email, phone, training_level FROM students WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await?;

    tracing::info!("Created student {} ({})", student.name, student.id);
    Ok((StatusCode::CREATED, Json(student.into())))
}
