//! API response handling and error types
//! 
//! This module provides standardized response types and error handling
//! for the API endpoints. It includes structures for successful responses
//! and error responses, along with helper methods for creating them.

use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::response::status;
use rocket::request::Request;
use rocket::response::{self, Responder};
use serde::{Serialize, Deserialize};
use std::fmt::Debug;

/// Standard API response wrapper for successful operations
/// 
/// # Type Parameters
/// * `T` - The type of data being returned
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Indicates if the operation was successful
    pub success: bool,
    /// Optional message providing additional context
    pub message: Option<String>,
    /// The actual data being returned
    pub data: Option<T>,
}

/// Error response structure for API failures
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Always false for error responses
    pub success: bool,
    /// Error message describing what went wrong
    pub message: String,
    /// HTTP status code for the error
    #[serde(skip)]
    status: Status,
}

impl<T> ApiResponse<T> {
    /// Creates a new successful API response
    /// 
    /// # Arguments
    /// * `data` - The data to be included in the response
    /// 
    /// # Returns
    /// * Json wrapped ApiResponse with the provided data
    pub fn success(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            message: None,
            data: Some(data),
        })
    }
}

impl ApiError {
    /// Creates a new API error response
    /// 
    /// # Arguments
    /// * `message` - Error message describing what went wrong
    /// * `status` - HTTP status code for the error
    /// 
    /// # Returns
    /// * New ApiError instance
    pub fn new(message: impl Into<String>, status: Status) -> Self {
        Self {
            success: false,
            message: message.into(),
            status,
        }
    }
}

/// Implementation of Rocket's Responder trait for ApiError
/// 
/// This allows ApiError to be returned directly from route handlers
/// while maintaining proper HTTP status codes and JSON formatting
impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let status = self.status;
        let json = Json(self);
        status::Custom(status, json).respond_to(req)
    }
}

/// Type alias for the standard result type used by API endpoints
/// 
/// This type combines ApiResponse for success cases and ApiError for failures
pub type ApiResult<T> = Result<Json<ApiResponse<T>>, ApiError>;
