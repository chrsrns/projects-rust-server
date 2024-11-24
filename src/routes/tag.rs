use rocket::http::Status;
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_db_pools::Connection;

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::db::tag::{ProjectToTechTag, Tag};
use crate::db::tag_category_join::TagCategory;
use crate::Db;

#[get("/api/tags")]
pub async fn tags(db: Connection<Db>) -> Result<Json<Vec<Tag>>, rocket::http::Status> {
    match Tag::get_all(db).await {
        Ok(results) => Ok(Json(results)),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[post("/api/tag", data = "<tag>", format = "json")]
pub async fn create_tag(
    db: Connection<Db>,
    tag: Json<Tag>,
) -> Result<Created<Json<Tag>>, rocket::http::Status> {
    let tag_deser = Tag {
        id: None,
        text: tag.text.clone(),
    };
    let result = tag_deser.add_or_get(db).await;
    match result {
        Ok(result) => Ok(Created::new("/").body(Json(result))),
        Err(e) => {
            eprintln!("Error creating or retrieving tag: {}", e);
            Err(rocket::http::Status::InternalServerError)
        }
    }
}

#[get("/api/tags/by-project/<id>")]
pub async fn tags_by_project(
    db: Connection<Db>,
    id: i32,
) -> Result<Json<Vec<Tag>>, rocket::http::Status> {
    let results = match Tag::get_tags_by_project(db, &id).await {
        Ok(result) => result,
        Err(_error) => return Err(rocket::http::Status::UnprocessableEntity),
    };
    Ok(Json(results))
}

#[get("/api/tags-by-category/<category>")]
pub async fn tags_by_category(
    db: Connection<Db>,
    category: String,
) -> Result<Json<Vec<Tag>>, rocket::http::Status> {
    let category = match TagCategory::from_str(category.as_str()) {
        Ok(result) => result,
        Err(_error) => return Err(rocket::http::Status::UnprocessableEntity),
    };
    let results = match Tag::get_tags_by_category(db, category).await {
        Ok(result) => result,
        Err(_error) => return Err(rocket::http::Status::InternalServerError),
    };
    Ok(Json(results))
}

#[derive(Serialize, Deserialize)]
pub struct TagAndCategoryData {
    pub tag: Tag,
    pub category: TagCategory,
}

#[post("/api/tag_category", data = "<data>", format = "json")]
pub async fn tag_category(db: Connection<Db>, data: Json<TagAndCategoryData>) -> Status {
    let result = data.tag.add_category(db, &data.category).await;

    match result {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError,
    }
}

#[post("/api/tag_project", data = "<data>", format = "json")]
pub async fn tag_project(db: Connection<Db>, data: Json<ProjectToTechTag>) -> Status {
    let result = data.add(db).await;

    match result {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError,
    }
}