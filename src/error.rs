use std::string::FromUtf8Error;

use actix_web::{
    http::{self, uri::InvalidUri},
    HttpResponse, ResponseError,
};

#[derive(Debug, thiserror::Error)]
pub enum CustomError {
    #[error("{0}")]
    VarError(#[from] std::env::VarError),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    Base64Error(#[from] base64ct::Error),
    #[error("{0}")]
    JwtError(#[from] web_push_native::jwt_simple::Error),
    #[error("{0}")]
    InvalidUri(#[from] InvalidUri),
    #[error("{0}")]
    ElipticCurveError(#[from] web_push_native::p256::elliptic_curve::Error),
    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("{0}")]
    WebPushError(#[from] web_push_native::Error),
    #[error("{0}")]
    HyperError(#[from] hyper::Error),
    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),
}

impl ResponseError for CustomError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
    fn error_response(&self) -> actix_web::HttpResponse {
        return HttpResponse::InternalServerError().json("Internal Server Error");
    }
}
