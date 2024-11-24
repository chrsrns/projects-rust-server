#[macro_use]
extern crate rocket;

use db::blog_item::{BlogItem, Content};
use db::project_item::{DescItem, ProjectItem};
use db::tag::{ProjectToTechTag, Tag};
use db::tag_category_join::TagCategory;
use db::user::User;
use rocket::http::{Method, Status};
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::{fairing, Build};
use rocket::Rocket;
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_db_pools::{Connection, Database};
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use sqlx::Either::{Left, Right};
use std::str::FromStr;

mod db;
mod routes;

#[derive(Database)]
#[database("sqlx")]
pub struct Db(sqlx::PgPool);

pub async fn init_database(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("src/db/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

#[get("/online-shopping-solidjs/assets/<file..>")]
async fn shop_solidjs(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("online-shopping-solidjs/assets/").join(file))
        .await
        .ok()
}

#[get("/online-shopping-solidjs/<_..>", rank = 2)]
async fn files() -> Option<NamedFile> {
    NamedFile::open(Path::new("online-shopping-solidjs/index.html"))
        .await
        .ok()
}

#[post("/api/user", data = "<user>", format = "json")]
async fn create_user(
    db: Connection<Db>,
    user: Json<User>,
) -> Result<Created<Json<User>>, Status> {
    let user_deser = User {
        id: None,
        username: user.username.clone(),
        upassword: user.upassword.clone(),
        email: user.email.clone(),
    };

    match user_deser.add(db).await {
        Ok(result) => {
            let resulted_id = result.id.expect("This shouldn't have happened, but it did");
            let user = Json(User {
                id: Some(resulted_id),
                username: user.username.clone(),
                upassword: user.upassword.clone(),
                email: user.email.clone(),
            });
            Ok(Created::new("/").body(user))
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/api/blogs")]
async fn blogs(db: Connection<Db>) -> Result<Json<Vec<BlogItem>>, rocket::http::Status> {
    match BlogItem::get_all(db).await {
        Ok(results) => Ok(Json(results)),
        Err(error) => {
            eprintln!("Error: Could not fetch blog items: {}", error);
            Err(rocket::http::Status::InternalServerError)
        }
    }
}

#[post("/api/blog", data = "<blog_item>", format = "json")]
async fn create_blog(
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

#[get("/api/blog-content/<id>")]
async fn blog_contents(
    db: Connection<Db>,
    id: i32,
) -> Result<Json<Vec<Content>>, rocket::http::Status> {
    match Content::get_all_from_blog(db, id).await {
        Ok(results) => Ok(Json(results)),
        Err(error) => {
            eprintln!("Error: Could not fetch blog content: {}", error);
            Err(rocket::http::Status::InternalServerError)
        }
    }
}

#[get("/api/projects")]
async fn projects(db: Connection<Db>) -> Result<Json<Vec<ProjectItem>>, rocket::http::Status> {
    let results = match ProjectItem::get_all(db).await {
        Ok(results) => results,
        Err(error) => {
            eprintln!("Error: Could not fetch projects: {}", error);
            return Err(rocket::http::Status::InternalServerError);
        }
    };
    Ok(Json(results))
}

#[get("/api/projects-by-tag/<tag_id>")]
async fn projects_by_tag(
    db: Connection<Db>,
    tag_id: i32,
) -> Result<Json<Vec<ProjectItem>>, rocket::http::Status> {
    let results = match ProjectItem::get_projects_by_tag(db, tag_id).await {
        Ok(results) => results,
        Err(error) => {
            eprintln!("Error: Could not fetch projects by tag: {}", error);
            return Err(rocket::http::Status::InternalServerError);
        }
    };
    Ok(Json(results))
}

#[post("/api/project", data = "<project_item>", format = "json")]
async fn create_project_item(
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

#[derive(Serialize, Deserialize)]
struct ProjectToTagsData {
    pub project: ProjectItem,
    pub tags: Vec<Tag>,
}
#[post("/api/project/tag", data = "<data>", format = "json")]
async fn add_tags_to_project(db: Connection<Db>, data: Json<ProjectToTagsData>) -> Status {
    let project_item = &data.project;
    // Converts vec of values to vec of references
    let tags = data.tags.iter().collect();

    match project_item.add_tag(db, tags).await {
        Ok(_result) => Status::Ok,
        Err(_error) => Status::UnprocessableEntity,
    }
}

#[get("/api/project_descs/<id>")]
async fn project_descs(
    db: Connection<Db>,
    id: i32,
) -> Result<Json<Vec<DescItem>>, rocket::http::Status> {
    match DescItem::get_all_from_project(db, id).await {
        Ok(results) => Ok(Json(results)),
        Err(error) => {
            eprintln!("Error: Could not fetch project descriptions: {}", error);
            Err(rocket::http::Status::InternalServerError)
        }
    }
}

#[post("/api/project_desc", data = "<project_desc>", format = "json")]
async fn create_project_desc(
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
async fn create_project_desc_many(
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

#[get("/api/tags")]
async fn tags(db: Connection<Db>) -> Result<Json<Vec<Tag>>, rocket::http::Status> {
    match Tag::get_all(db).await {
        Ok(results) => Ok(Json(results)),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[post("/api/tag", data = "<tag>", format = "json")]
async fn create_tag(
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
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[get("/api/tags-by-category/<category>")]
async fn tags_by_category(
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

#[get("/api/tags/by-project/<id>")]
async fn tags_by_project(
    db: Connection<Db>,
    id: i32,
) -> Result<Json<Vec<Tag>>, rocket::http::Status> {
    let results = match Tag::get_tags_by_project(db, &id).await {
        Ok(result) => result,
        Err(_error) => return Err(rocket::http::Status::UnprocessableEntity),
    };
    Ok(Json(results))
}

#[derive(Serialize, Deserialize)]
struct TagAndCategoryData {
    pub tag: Tag,
    pub category: TagCategory,
}

#[post("/api/tag_category", data = "<data>", format = "json")]
async fn tag_category(db: Connection<Db>, data: Json<TagAndCategoryData>) -> Status {
    let result = data.tag.add_category(db, &data.category).await;

    match result {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError,
    }
}

#[post("/api/tag_project", data = "<data>", format = "json")]
async fn tag_project(db: Connection<Db>, data: Json<ProjectToTechTag>) -> Status {
    let result = data.add(db).await;

    match result {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError,
    }
}

#[get("/api/users")]
async fn users(db: Connection<Db>) -> Result<Json<Vec<User>>, rocket::http::Status> {
    match User::get_all_users(db).await {
        Ok(results) => Ok(Json(results)),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[launch]
async fn rocket() -> _ {
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    let rocket = rocket::build();
    let rocket = match init_database(rocket).await {
        Ok(rocket) => rocket,
        Err(r) => r,
    };

    rocket
        .attach(cors.to_cors().unwrap())
        .attach(Db::init())
        .mount(
            "/",
            routes![
                routes::static_files::solidjs_assets,
                routes::static_files::solidjs_index,
                routes::shop::shop_items,
                routes::shop::create_shop_item,
                routes::shop::shop_item_images,
                routes::shop::create_shop_item_image,
                routes::shop::shop_item_descs,
                routes::shop::create_shop_item_desc,
                routes::shop::create_shop_item_desc_many,
                blogs,
                blog_contents,
                create_blog,
                projects,
                projects_by_tag,
                tag_category,
                tag_project,
                add_tags_to_project,
                project_descs,
                create_project_item,
                create_project_desc,
                create_project_desc_many,
                tags,
                create_tag,
                tags_by_project,
                tags_by_category,
                users,
                create_user,
            ],
        )
}
