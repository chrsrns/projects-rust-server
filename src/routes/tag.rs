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

#[derive(Serialize, Deserialize)]
pub struct TagAndCategoryData {
    pub tag: Tag,
    pub category: TagCategory,
}

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