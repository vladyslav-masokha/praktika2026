use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crate::dto::ApiErrorResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Broker error: {0}")]
    Broker(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, client_message) = match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            
            AppError::Database(_) | AppError::Broker(_) | AppError::Internal(_) => {
                tracing::error!("Internal system error: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred. Please try again later.".to_string(),
                )
            }
        };

        let body = Json(ApiErrorResponse {
            message: client_message,
        });

        (status, body).into_response()
    }
}