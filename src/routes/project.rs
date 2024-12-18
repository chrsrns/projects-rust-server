//! Project management routes
//! 
//! This module handles all project-related API endpoints, including:
//! - Project CRUD operations
//! - Project description management
//! - Project-tag associations

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use sqlx::Either::{Left, Right};

use crate::db::project_item::{DescItem, ProjectItem};
use crate::db::tag::Tag;
use crate::Db;
use crate::api::{ApiResponse, ApiResult, ApiError};

/// Retrieves all projects
/// 
/// # Returns
/// * `ApiResult<Vec<ProjectItem>>` - List of all projects on success
#[get("/api/projects")]
pub async fn projects(db: Connection<Db>) -> ApiResult<Vec<ProjectItem>> {
    match ProjectItem::get_all(db).await {
        Ok(results) => Ok(ApiResponse::success(results)),
        Err(_error) => {
            Err(ApiError::new("Failed to fetch project items", Status::InternalServerError))
        }
    }
}

/// Retrieves all projects associated with a specific tag
/// 
/// # Arguments
/// * `db` - Database connection
/// * `tag_id` - ID of the tag to filter projects by
/// 
/// # Returns
/// * `ApiResult<Vec<ProjectItem>>` - List of projects with the specified tag
#[get("/api/projects-by-tag/<tag_id>")]
pub async fn projects_by_tag(
    db: Connection<Db>,
    tag_id: i32,
) -> ApiResult<Vec<ProjectItem>> {
    match ProjectItem::get_projects_by_tag(db, tag_id).await {
        Ok(results) => Ok(ApiResponse::success(results)),
        Err(_error) => {
            Err(ApiError::new("Failed to fetch projects by tag", Status::InternalServerError))
        }
    }
}

/// Data structure for associating multiple tags with a project
#[derive(Serialize, Deserialize)]
pub struct ProjectToTagsData {
    /// The project to add tags to
    pub project: ProjectItem,
    /// List of tags to associate with the project
    pub tags: Vec<Tag>,
}

/// Associates multiple tags with a project
/// 
/// # Arguments
/// * `db` - Database connection
/// * `data` - Project and tags data
/// 
/// # Returns
/// * `ApiResult<()>` - Success or failure of the operation
#[post("/api/project/tag", data = "<data>", format = "json")]
pub async fn add_tags_to_project(db: Connection<Db>, data: Json<ProjectToTagsData>) -> ApiResult<()> {
    let project_item = &data.project;
    let tags = data.tags.iter().collect();

    match project_item.add_tag(db, tags).await {
        Ok(_result) => Ok(ApiResponse::success(())),
        Err(_error) => {
            Err(ApiError::new(
                "Failed to add tags to project",
                Status::InternalServerError
            ))
        }
    }
}

/// Retrieves all descriptions for a specific project
/// 
/// # Arguments
/// * `db` - Database connection
/// * `id` - Project ID
/// 
/// # Returns
/// * `ApiResult<Vec<DescItem>>` - List of project descriptions
#[get("/api/project_descs/<id>")]
pub async fn project_descs(
    db: Connection<Db>,
    id: i32,
) -> ApiResult<Vec<DescItem>> {
    match DescItem::get_all_from_project(db, id).await {
        Ok(results) => Ok(ApiResponse::success(results)),
        Err(_error) => {
            Err(ApiError::new(
                "Failed to fetch project descriptions",
                Status::InternalServerError
            ))
        }
    }
}

/// Creates a new project
/// 
/// # Arguments
/// * `db` - Database connection
/// * `project_item` - Project data to create
/// 
/// # Returns
/// * `ApiResult<ProjectItem>` - Created project with assigned ID
#[post("/api/project", data = "<project_item>", format = "json")]
pub async fn create_project_item(
    db: Connection<Db>,
    project_item: Json<ProjectItem>,
) -> ApiResult<ProjectItem> {
    let project_item_deser = ProjectItem {
        id: None,
        title: project_item.title.clone(),
        thumbnail_img_link: project_item.thumbnail_img_link.clone(),
        desc: project_item.desc.clone(),
    };

    match project_item_deser.add(db).await {
        Ok(result) => {
            match result.id {
                Some(_) => Ok(ApiResponse::success(result)),
                None => Err(ApiError::new(
                    "Failed to create project: No ID returned",
                    Status::InternalServerError
                ))
            }
        }
        Err(_error) => {
            Err(ApiError::new(
                "Failed to create project",
                Status::InternalServerError
            ))
        }
    }
}

/// Creates a new project description
/// 
/// # Arguments
/// * `db` - Database connection
/// * `project_desc` - Project description data
/// 
/// # Returns
/// * `ApiResult<DescItem>` - Created description with assigned ID
#[post("/api/project_desc", data = "<project_desc>", format = "json")]
pub async fn create_project_desc(
    db: Connection<Db>,
    project_desc: Json<DescItem>,
) -> ApiResult<DescItem> {
    let project_desc_deser = DescItem {
        id: None,
        project_id: project_desc.project_id,
        content: project_desc.content.clone(),
    };

    let result = match project_desc_deser.add(db).await {
        Ok(query_result) => query_result,
        Err(error) => match error {
            Left(_) => {
                return Err(ApiError::new(
                    "Failed to create project description: Invalid input",
                    Status::BadRequest
                ));
            }
            Right(_error) => {
                return Err(ApiError::new(
                    "Failed to create project description",
                    Status::InternalServerError
                ));
            }
        },
    };

    match result.id {
        Some(_) => Ok(ApiResponse::success(result)),
        None => Err(ApiError::new(
            "Failed to create project description: No ID returned",
            Status::InternalServerError
        )),
    }
}

/// Creates multiple project descriptions in a single transaction
/// 
/// # Arguments
/// * `db` - Database connection
/// * `project_descs` - List of project descriptions to create
/// 
/// # Returns
/// * `ApiResult<()>` - Success or failure of the operation
#[post("/api/project_desc_many", data = "<project_descs>", format = "json")]
pub async fn create_project_desc_many(
    mut db: Connection<Db>,
    project_descs: Json<Vec<DescItem>>,
) -> ApiResult<()> {
    let mut tx = match (*db).begin().await {
        Ok(tx) => tx,
        Err(_error) => {
            return Err(ApiError::new(
                "Failed to create project descriptions",
                Status::InternalServerError
            ));
        }
    };

    for project_desc in project_descs.iter() {
        let result = project_desc.add_tx(&mut tx).await;
        match result {
            Err(error) => {
                if error.is_left() {
                    return Err(ApiError::new(
                        "Failed to create project descriptions: Invalid input",
                        Status::BadRequest
                    ));
                } else {
                    return Err(ApiError::new(
                        "Failed to create project descriptions",
                        Status::InternalServerError
                    ));
                }
            }
            _ => continue,
        }
    }

    match tx.commit().await {
        Ok(_) => Ok(ApiResponse::success(())),
        Err(_error) => {
            Err(ApiError::new(
                "Failed to create project descriptions",
                Status::InternalServerError
            ))
        }
    }
}