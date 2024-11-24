pub use rocket::response::status::Created;
pub use rocket::serde::json::Json;
pub use rocket::{get, post};
pub use rocket_db_pools::Connection;

pub use crate::db::blog_item::{BlogItem, Content};
pub use crate::Db;

#[get("/api/blogs")]
pub async fn blogs(db: Connection<Db>) -> Result<Json<Vec<BlogItem>>, rocket::http::Status> {
    match BlogItem::get_all(db).await {
        Ok(results) => Ok(Json(results)),
        Err(error) => {
            eprintln!("Error: Could not get blog items: {}", error);
            Err(rocket::http::Status::InternalServerError)
        }
    }
}

#[get("/api/blog-content/<id>")]
pub async fn blog_contents(
    db: Connection<Db>,
    id: i32,
) -> Result<Json<Vec<Content>>, rocket::http::Status> {
    match Content::get_all_from_blog(db, id).await {
        Ok(results) => Ok(Json(results)),
        Err(error) => {
            eprintln!("Error: Could not get blog contents: {}", error);
            Err(rocket::http::Status::InternalServerError)
        }
    }
}

#[post("/api/blog", data = "<blog_item>", format = "json")]
pub async fn create_blog(
    db: Connection<Db>,
    blog_item: Json<BlogItem>,
) -> Result<Created<Json<BlogItem>>, rocket::http::Status> {
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
                Ok(Created::new("/").body(Json(query_result)))
            } else {
                Err(rocket::http::Status::InternalServerError)
            }
        }
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}