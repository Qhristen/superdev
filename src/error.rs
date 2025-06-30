use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub enum ApiError {
    InternalError(String),
    BadRequest(String),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ApiError::InternalError(msg) => write!(f, "Internal server error: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let error_msg = self.to_string();
        let status_code = match self {
            ApiError::InternalError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest(_) => actix_web::http::StatusCode::BAD_REQUEST,
        };

        HttpResponse::build(status_code).json(ErrorResponse {
            success: false,
            error: error_msg,
        })
    }
}
