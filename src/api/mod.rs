use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::response::status;
use rocket::request::Request;
use rocket::response::{self, Responder};
use serde::{Serialize, Deserialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub success: bool,
    pub message: String,
    #[serde(skip)]
    status: Status,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            message: None,
            data: Some(data),
        })
    }
}

impl ApiError {
    pub fn new(message: impl Into<String>, status: Status) -> Self {
        Self {
            success: false,
            message: message.into(),
            status,
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let status = self.status;
        let json = Json(self);
        status::Custom(status, json).respond_to(req)
    }
}

pub type ApiResult<T> = Result<Json<ApiResponse<T>>, ApiError>;
