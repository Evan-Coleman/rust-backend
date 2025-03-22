//! User API endpoints
//!
//! This module provides API endpoints for user management.

use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::router::AppState;
use crate::repository::models::UserRole;
use crate::services::{ServiceError, UserService};

/// API response for user operations
#[derive(Debug, Serialize)]
pub struct UserResponse {
    /// User ID
    pub id: Uuid,

    /// Username
    pub username: String,

    /// Email
    pub email: String,

    /// Full name
    pub full_name: Option<String>,

    /// Whether the user is active
    pub is_active: bool,

    /// User role
    pub role: String,

    /// When the user was created
    pub created_at: String,

    /// When the user was last updated
    pub updated_at: String,
}

impl From<crate::repository::User> for UserResponse {
    fn from(user: crate::repository::User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            full_name: user.full_name,
            is_active: user.is_active,
            role: user.role.to_string(),
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}

/// Create user request
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    /// Username
    pub username: String,

    /// Email
    pub email: String,

    /// Full name (optional)
    pub full_name: Option<String>,

    /// User role (optional)
    pub role: Option<String>,
}

/// Update user request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    /// Email (optional)
    pub email: Option<String>,

    /// Full name (optional)
    pub full_name: Option<String>,

    /// Whether the user is active (optional)
    pub is_active: Option<bool>,

    /// User role (optional)
    pub role: Option<String>,
}

/// Configure user routes
pub fn configure() -> Router<AppState> {
    Router::new()
        .route("/users", get(get_all_users))
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
}

/// Map service errors to HTTP status codes
fn map_service_error(err: ServiceError) -> (StatusCode, String) {
    match err {
        ServiceError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),
        ServiceError::UsernameExists => {
            (StatusCode::CONFLICT, "Username already exists".to_string())
        }
        ServiceError::EmailExists => (StatusCode::CONFLICT, "Email already exists".to_string()),
        ServiceError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        ),
    }
}

/// Get all users
async fn get_all_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, (StatusCode, String)> {
    // Get user service from app state
    let user_service = get_user_service(state)?;

    // Get all users from service
    let users = user_service
        .get_all_users()
        .await
        .map_err(map_service_error)?;

    // Map users to response format
    let responses = users.into_iter().map(UserResponse::from).collect();

    Ok(Json(responses))
}

/// Get a user by ID
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    // Get user service from app state
    let user_service = get_user_service(state)?;

    // Get user from service
    let user = user_service
        .get_user_by_id(id)
        .await
        .map_err(map_service_error)?;

    // Map user to response format
    let response = UserResponse::from(user);

    Ok(Json(response))
}

/// Create a new user
async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, String)> {
    // Get user service from app state
    let user_service = get_user_service(state)?;

    // Map role string to enum if provided
    let role = match request.role {
        Some(role_str) => match role_str.as_str() {
            "admin" => Some(UserRole::Admin),
            "user" => Some(UserRole::User),
            "readonly" => Some(UserRole::ReadOnly),
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("Invalid role: {}", role_str),
                ));
            }
        },
        None => None,
    };

    // Create DTO from request
    let create_dto = crate::services::user::CreateUserDto {
        username: request.username,
        email: request.email,
        full_name: request.full_name,
        role,
    };

    // Create user via service
    let user = user_service
        .create_user(create_dto)
        .await
        .map_err(map_service_error)?;

    // Map user to response format
    let response = UserResponse::from(user);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Update a user
async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    // Get user service from app state
    let user_service = get_user_service(state)?;

    // Map role string to enum if provided
    let role = match request.role {
        Some(role_str) => match role_str.as_str() {
            "admin" => Some(UserRole::Admin),
            "user" => Some(UserRole::User),
            "readonly" => Some(UserRole::ReadOnly),
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("Invalid role: {}", role_str),
                ));
            }
        },
        None => None,
    };

    // Create DTO from request
    let update_dto = crate::services::user::UpdateUserDto {
        email: request.email,
        full_name: request.full_name,
        is_active: request.is_active,
        role,
    };

    // Update user via service
    let user = user_service
        .update_user(id, update_dto)
        .await
        .map_err(map_service_error)?;

    // Map user to response format
    let response = UserResponse::from(user);

    Ok(Json(response))
}

/// Delete a user
async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Get user service from app state
    let user_service = get_user_service(state)?;

    // Delete user via service
    let deleted = user_service
        .delete_user(id)
        .await
        .map_err(map_service_error)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete user".to_string(),
        ))
    }
}

/// Helper function to get the user service from app state
fn get_user_service(state: AppState) -> Result<Arc<UserService>, (StatusCode, String)> {
    // Get the database pool from app state
    let db_pool = match state.db_pool {
        Some(ref pool) => pool.clone(),
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database not configured".to_string(),
            ));
        }
    };

    // Create user repository
    let user_repo = Arc::new(crate::repository::UserRepository::new(db_pool));

    // Create user service
    let user_service = Arc::new(UserService::new(user_repo));

    Ok(user_service)
}
