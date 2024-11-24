//! Tag management routes module
//! 
//! This module provides endpoints for managing tags, including creating, retrieving,
//! and associating tags with projects and categories.

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_db_pools::Connection;

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::db::tag::{ProjectToTechTag, Tag};
use crate::db::tag_category_join::TagCategory;
use crate::Db;
use crate::api::{ApiResponse, ApiResult, ApiError};

/// Retrieves all tags from the database
/// 
/// # Returns
/// - `ApiResult<Vec<Tag>>`: A list of all tags on success
/// - `ApiError`: If database operation fails
#[get("/api/tags")]
pub async fn tags(db: Connection<Db>) -> ApiResult<Vec<Tag>> {
    match Tag::get_all(db).await {
        Ok(results) => Ok(ApiResponse::success(results)),
        Err(_) => Err(ApiError::new(
            "Failed to fetch tags",
            Status::InternalServerError
        )),
    }
}

/// Creates a new tag or returns an existing one if it already exists
/// 
/// # Arguments
/// * `db` - Database connection
/// * `tag` - The tag data to create
/// 
/// # Returns
/// - `ApiResult<Tag>`: The created or existing tag
/// - `ApiError`: If tag creation fails
#[post("/api/tag", data = "<tag>", format = "json")]
pub async fn create_tag(
    db: Connection<Db>,
    tag: Json<Tag>,
) -> ApiResult<Tag> {
    let tag_deser = Tag {
        id: None,
        text: tag.text.clone(),
    };
    match tag_deser.add_or_get(db).await {
        Ok(result) => Ok(ApiResponse::success(result)),
        Err(_) => Err(ApiError::new(
            "Failed to create tag",
            Status::InternalServerError
        )),
    }
}

/// Retrieves all tags associated with a specific project
/// 
/// # Arguments
/// * `db` - Database connection
/// * `id` - Project ID
/// 
/// # Returns
/// - `ApiResult<Vec<Tag>>`: List of tags associated with the project
/// - `ApiError`: If fetching fails
#[get("/api/tags/by-project/<id>")]
pub async fn tags_by_project(
    db: Connection<Db>,
    id: i32,
) -> ApiResult<Vec<Tag>> {
    match Tag::get_tags_by_project(db, &id).await {
        Ok(results) => Ok(ApiResponse::success(results)),
        Err(_) => Err(ApiError::new(
            "Failed to fetch tags by project",
            Status::InternalServerError
        )),
    }
}

/// Retrieves all tags belonging to a specific category
/// 
/// # Arguments
/// * `db` - Database connection
/// * `category` - Category name as string
/// 
/// # Returns
/// - `ApiResult<Vec<Tag>>`: List of tags in the category
/// - `ApiError`: If category is invalid or fetching fails
#[get("/api/tags-by-category/<category>")]
pub async fn tags_by_category(
    db: Connection<Db>,
    category: String,
) -> ApiResult<Vec<Tag>> {
    let category = match TagCategory::from_str(category.as_str()) {
        Ok(result) => result,
        Err(_error) => return Err(ApiError::new(
            "Invalid category",
            Status::UnprocessableEntity
        )),
    };
    match Tag::get_tags_by_category(db, category).await {
        Ok(results) => Ok(ApiResponse::success(results)),
        Err(_) => Err(ApiError::new(
            "Failed to fetch tags by category",
            Status::InternalServerError
        )),
    }
}

/// Data structure for associating a tag with a category
#[derive(Serialize, Deserialize)]
pub struct TagAndCategoryData {
    /// The tag to be associated
    pub tag: Tag,
    /// The category to associate the tag with
    pub category: TagCategory,
}

/// Associates a tag with a category
/// 
/// # Arguments
/// * `db` - Database connection
/// * `data` - Tag and category association data
/// 
/// # Returns
/// - `ApiResult<String>`: Success message
/// - `ApiError`: If association fails
#[post("/api/tag_category", data = "<data>", format = "json")]
pub async fn tag_category(db: Connection<Db>, data: Json<TagAndCategoryData>) -> ApiResult<String> {
    let result = data.tag.add_category(db, &data.category).await;

    match result {
        Ok(_) => Ok(ApiResponse::success("Tag category added successfully".to_string())),
        Err(_) => Err(ApiError::new(
            "Failed to add tag category",
            Status::InternalServerError
        )),
    }
}

/// Associates a tag with a project
/// 
/// # Arguments
/// * `db` - Database connection
/// * `data` - Project and tag association data
/// 
/// # Returns
/// - `ApiResult<String>`: Success message
/// - `ApiError`: If association fails
#[post("/api/tag_project", data = "<data>", format = "json")]
pub async fn tag_project(db: Connection<Db>, data: Json<ProjectToTechTag>) -> ApiResult<String> {
    let result = data.add(db).await;

    match result {
        Ok(_) => Ok(ApiResponse::success("Tag project added successfully".to_string())),
        Err(_) => Err(ApiError::new(
            "Failed to add tag project",
            Status::InternalServerError
        )),
    }
}