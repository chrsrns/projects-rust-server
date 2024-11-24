use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_db_pools::Connection;

use crate::db::user::User;
use crate::Db;
use crate::api::{ApiResponse, ApiResult, ApiError};

#[get("/api/users")]
pub async fn users(db: Connection<Db>) -> ApiResult<Vec<User>> {
    match User::get_all_users(db).await {
        Ok(results) => Ok(ApiResponse::success(results)),
        Err(_) => Err(ApiError::new(
            "Failed to fetch users",
            Status::InternalServerError
        )),
    }
}

#[post("/api/user", data = "<user>", format = "json")]
pub async fn create_user(
    db: Connection<Db>,
    user: Json<User>,
) -> ApiResult<User> {
    let user_deser = User {
        id: None,
        username: user.username.clone(),
        upassword: user.upassword.clone(),
        email: user.email.clone(),
    };

    match user_deser.add(db).await {
        Ok(result) => {
            match result.id {
                Some(_) => Ok(ApiResponse::success(result)),
                None => Err(ApiError::new(
                    "Failed to create user: No ID returned",
                    Status::NotFound
                )),
            }
        }
        Err(error) => {
            // Check if it's a unique constraint violation (e.g., duplicate username/email)
            if error.to_string().contains("unique constraint") {
                Err(ApiError::new(
                    "Username or email already exists",
                    Status::Conflict
                ))
            } else {
                Err(ApiError::new(
                    "Failed to create user",
                    Status::InternalServerError
                ))
            }
        }
    }
}