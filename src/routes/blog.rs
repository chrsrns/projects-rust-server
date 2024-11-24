use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_db_pools::Connection;

pub use crate::db::blog_item::{BlogItem, Content};
pub use crate::Db;
pub use crate::api::{ApiResponse, ApiResult, ApiError};

#[get("/api/blogs")]
pub async fn blogs(db: Connection<Db>) -> ApiResult<Vec<BlogItem>> {
    match BlogItem::get_all(db).await {
        Ok(results) => Ok(ApiResponse::success(results)),
        Err(_error) => {
            Err(ApiError::new(
                "Failed to fetch blog items",
                Status::InternalServerError
            ))
        }
    }
}

#[get("/api/blog-content/<id>")]
pub async fn blog_contents(
    db: Connection<Db>,
    id: i32,
) -> ApiResult<Vec<Content>> {
    match Content::get_all_from_blog(db, id).await {
        Ok(results) => Ok(ApiResponse::success(results)),
        Err(_error) => {
            Err(ApiError::new(
                "Failed to fetch blog contents",
                Status::InternalServerError
            ))
        }
    }
}

#[post("/api/blog", data = "<blog_item>", format = "json")]
pub async fn create_blog(
    db: Connection<Db>,
    blog_item: Json<BlogItem>,
) -> ApiResult<BlogItem> {
    let blog_item_deser = BlogItem {
        id: None,
        blog_title: blog_item.blog_title.clone(),
        header_img: blog_item.header_img.clone(),
        content: blog_item.content.clone(),
    };
    let result = blog_item_deser.add(db).await;
    
    match result {
        Ok(query_result) => {
            if query_result.id.is_some() {
                Ok(ApiResponse::success(query_result))
            } else {
                Err(ApiError::new(
                    "Failed to create blog item: No ID returned",
                    Status::InternalServerError
                ))
            }
        }
        Err(_error) => {
            Err(ApiError::new(
                "Failed to create blog item",
                Status::InternalServerError
            ))
        }
    }
}