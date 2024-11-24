use rocket::http::Status;
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use sqlx::Either::{Left, Right};

use crate::db::project_item::{DescItem, ProjectItem};
use crate::db::tag::Tag;
use crate::Db;

#[get("/api/projects")]
pub async fn projects(db: Connection<Db>) -> Result<Json<Vec<ProjectItem>>, rocket::http::Status> {
    let results = match ProjectItem::get_all(db).await {
        Ok(results) => results,
        Err(error) => {
            eprintln!("Error: Could not get project items: {}", error);
            return Err(rocket::http::Status::InternalServerError);
        }
    };
    Ok(Json(results))
}

#[get("/api/projects-by-tag/<tag_id>")]
pub async fn projects_by_tag(
    db: Connection<Db>,
    tag_id: i32,
) -> Result<Json<Vec<ProjectItem>>, rocket::http::Status> {
    let results = match ProjectItem::get_projects_by_tag(db, tag_id).await {
        Ok(results) => results,
        Err(error) => {
            eprintln!("Error: Could not get project items by tag: {}", error);
            return Err(rocket::http::Status::InternalServerError);
        }
    };
    Ok(Json(results))
}

#[derive(Serialize, Deserialize)]
pub struct ProjectToTagsData {
    pub project: ProjectItem,
    pub tags: Vec<Tag>,
}

#[post("/api/project/tag", data = "<data>", format = "json")]
pub async fn add_tags_to_project(db: Connection<Db>, data: Json<ProjectToTagsData>) -> Status {
    let project_item = &data.project;
    // Converts vec of values to vec of references
    let tags = data.tags.iter().collect();

    match project_item.add_tag(db, tags).await {
        Ok(_result) => Status::Ok,
        Err(_error) => Status::UnprocessableEntity,
    }
}

#[get("/api/project_descs/<id>")]
pub async fn project_descs(
    db: Connection<Db>,
    id: i32,
) -> Result<Json<Vec<DescItem>>, rocket::http::Status> {
    match DescItem::get_all_from_project(db, id).await {
        Ok(results) => Ok(Json(results)),
        Err(error) => {
            eprintln!("Error: Could not get project descriptions: {}", error);
            Err(rocket::http::Status::InternalServerError)
        }
    }
}

#[post("/api/project", data = "<project_item>", format = "json")]
pub async fn create_project_item(
    db: Connection<Db>,
    project_item: Json<ProjectItem>,
) -> Result<Created<Json<ProjectItem>>, rocket::http::Status> {
    let project_item_deser = ProjectItem {
        id: None,
        title: project_item.title.clone(),
        thumbnail_img_link: project_item.thumbnail_img_link.clone(),
        desc: project_item.desc.clone(),
    };
    let result = match project_item_deser.add(db).await {
        Ok(result) => result,
        Err(error) => {
            eprintln!("Error: Could not add project item: {}", error);
            return Err(rocket::http::Status::InternalServerError);
        }
    };

    match result.id {
        Some(_) => Ok(Created::new("/").body(Json(result))),
        None => {
            eprintln!("Error: Could not add project item: Row not found for the given id");
            Err(rocket::http::Status::InternalServerError)
        }
    }
}

#[post("/api/project_desc", data = "<project_desc>", format = "json")]
pub async fn create_project_desc(
    db: Connection<Db>,
    project_desc: Json<DescItem>,
) -> Result<Created<Json<DescItem>>, rocket::http::Status> {
    let project_desc_deser = DescItem {
        id: None,
        project_id: project_desc.project_id,
        content: project_desc.content.clone(),
    };
    let result = match project_desc_deser.add(db).await {
        Ok(query_result) => query_result,
        Err(error) => match error {
            Left(_) => {
                return Err(rocket::http::Status::BadRequest);
            }
            Right(_) => {
                return Err(rocket::http::Status::InternalServerError);
            }
        },
    };
    match result.id {
        Some(_) => Ok(Created::new("/").body(Json(result))),
        None => Err(rocket::http::Status::NotFound),
    }
}

#[post("/api/project_desc", data = "<project_descs>", format = "json")]
pub async fn create_project_desc_many(
    mut db: Connection<Db>,
    project_descs: Json<Vec<DescItem>>,
) -> Result<Status, rocket::http::Status> {
    let mut tx = match (*db).begin().await {
        Ok(tx) => tx,
        Err(error) => {
            eprintln!("Error: Could not start transaction: {}", error);
            return Err(rocket::http::Status::InternalServerError);
        }
    };

    // TODO: Janky Error handling. Rewrite to be similar to many function somewhere above
    for project_desc in project_descs.iter() {
        let result = project_desc.add_tx(&mut tx).await;
        match result {
            Err(error) => {
                if error.is_left() {
                    return Err(rocket::http::Status::InternalServerError);
                } else {
                    return Ok(Status::UnprocessableEntity);
                }
            }
            _ => continue,
        }
    }
    match tx.commit().await {
        Ok(_) => Ok(Status::Ok),
        Err(error) => {
            eprintln!("Error: Could not commit transaction: {}", error);
            Err(rocket::http::Status::InternalServerError)
        }
    }
}