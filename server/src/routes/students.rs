use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use core::models::{Student, TrainingLevel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateStudentRequest {
    pub name: String,
    pub email: String,
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
) -> Result<Json<Vec<StudentResponse>>, StatusCode> {
    let students = sqlx::query_as::<_, Student>(
        "SELECT id, name, email, phone, training_level FROM students ORDER BY name"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch students: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(students.into_iter().map(StudentResponse::from).collect()))
}

pub async fn create_student(
    State(state): State<AppState>,
    Json(req): Json<CreateStudentRequest>,
) -> Result<(StatusCode, Json<StudentResponse>), StatusCode> {
    // Validate training level
    let training_level = match req.training_level.as_str() {
        "STUDENT_PILOT" => TrainingLevel::StudentPilot,
        "PRIVATE_PILOT" => TrainingLevel::PrivatePilot,
        "INSTRUMENT_RATED" => TrainingLevel::InstrumentRated,
        _ => return Err(StatusCode::BAD_REQUEST),
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
    .await
    .map_err(|e| {
        tracing::error!("Failed to create student: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Fetch created student
    let student = sqlx::query_as::<_, Student>(
        "SELECT id, name, email, phone, training_level FROM students WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch created student: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(student.into())))
}
