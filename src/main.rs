#[macro_use]
extern crate rocket;

use db::blog_item::{BlogItem, Content};
use db::project_item::{DescItem, ProjectItem};
use db::shop_item::{ShopImage, ShopItem, ShopItemDesc, ShopItemDescMany};
use db::tag::{ProjectToTechTag, Tag};
use db::tag_category_join::TagCategory;
use db::user::User;
use rocket::http::{Method, Status};
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::{fairing, Build};
use rocket::{fs::NamedFile, Rocket};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_db_pools::{Connection, Database};
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use sqlx::Either::{Left, Right};
use std::path::{Path, PathBuf};
use std::str::FromStr;

mod db;

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
) -> Result<Created<Json<User>>, rocket::http::Status> {
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
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[get("/api/shopitems")]
async fn shop_items(db: Connection<Db>) -> Result<Json<Vec<ShopItem>>, rocket::http::Status> {
    match ShopItem::get_all(db).await {
        Ok(results) => Ok(Json(results)),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}
#[post("/api/shopitem", data = "<shop_item>", format = "json")]
async fn create_shop_item(
    db: Connection<Db>,
    mut shop_item: Json<ShopItem>,
) -> Result<Created<Json<ShopItem>>, rocket::http::Status> {
    let shop_item_deser = ShopItem {
        id: None,
        iname: shop_item.iname.clone(),
        img_link: shop_item.img_link.clone(),
        price: shop_item.price,
    };
    let result = shop_item_deser.add(db).await;

    match result {
        Ok(query_result) => {
            if let Some(resulted_id) = query_result.id {
                shop_item.id = Some(resulted_id);
                Ok(Created::new("/").body(shop_item))
            } else {
                Err(rocket::http::Status::InternalServerError)
            }
        }
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[get("/api/shopitemimages/<id>")]
async fn shop_item_images(
    db: Connection<Db>,
    id: i32,
) -> Result<Json<Vec<ShopImage>>, rocket::http::Status> {
    match ShopImage::get_all_from_shop_item(db, id).await {
        Ok(results) => Ok(Json(results)),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[post("/api/shopitemimage", data = "<shop_item_image>", format = "json")]
async fn create_shop_item_image(
    db: Connection<Db>,
    shop_item_image: Json<ShopImage>,
) -> Result<Created<Json<ShopImage>>, rocket::http::Status> {
    let shop_item_desc_deser = ShopImage {
        id: None,
        shop_item_id: shop_item_image.shop_item_id,
        tooltip: shop_item_image.tooltip.clone(),
        img_link: shop_item_image.img_link.clone(),
    };
    let result = match shop_item_desc_deser.add(db).await {
        Ok(query_result) => query_result,
        Err(error) => match error {
            Left(sql_error) => {
                eprintln!("SQL Error occurred: {:?}", sql_error);
                return Err(rocket::http::Status::InternalServerError);
            }
            Right(_) => {
                eprintln!("Type not found for 'shop_item_id'");
                return Err(rocket::http::Status::BadRequest);
            }
        },
    };

    match result.id {
        Some(_) => Ok(Created::new("/").body(Json(result))),
        None => {
            eprintln!("Error: Row not found for the given shop item image.");
            Err(rocket::http::Status::NotFound)
        }
    }
}

#[get("/api/shopitemdescs/<id>")]
async fn shop_item_descs(
    db: Connection<Db>,
    id: i32,
) -> Result<Json<Vec<ShopItemDesc>>, rocket::http::Status> {
    match ShopItemDesc::get_all_from_shop_item(db, id).await {
        Ok(results) => Ok(Json(results)),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[post("/api/shopitemdesc", data = "<shop_item_desc>", format = "json")]
async fn create_shop_item_desc(
    db: Connection<Db>,
    shop_item_desc: Json<ShopItemDesc>,
) -> Result<Created<Json<ShopItemDesc>>, rocket::http::Status> {
    let shop_item_desc_deser = ShopItemDesc {
        id: None,
        shop_item_id: shop_item_desc.shop_item_id,
        content: shop_item_desc.content.clone(),
    };
    let result = match shop_item_desc_deser.add(db).await {
        Ok(query_result) => query_result,
        Err(error) => match error {
            Left(_) => {
                return Err(rocket::http::Status::InternalServerError);
            }
            Right(_) => {
                return Err(rocket::http::Status::BadRequest);
            }
        },
    };

    match result.id {
        Some(_) => Ok(Created::new("/").body(Json(result))),
        None => Err(rocket::http::Status::NotFound),
    }
}

#[post(
    "/api/shopitemdesc/many",
    data = "<shop_item_desc_many>",
    format = "json"
)]
async fn create_shop_item_desc_many(
    mut db: Connection<Db>,
    shop_item_desc_many: Json<ShopItemDescMany>,
) -> Result<Status, rocket::http::Status> {
    let mut tx = match (*db).begin().await {
        Ok(tx) => tx,
        Err(error) => {
            eprintln!("Error: Could not start transaction: {}", error);
            return Err(rocket::http::Status::InternalServerError);
        }
    };
    for content in &shop_item_desc_many.contents {
        match sqlx::query_as!(
            ShopItemDesc,
            "INSERT INTO shop_item_desc (shop_item_id, content) VALUES ($1, $2)",
            shop_item_desc_many.shop_item_id,
            content
        )
        .execute(&mut *tx)
        .await
        {
            Ok(_) => continue,
            Err(error) => {
                eprintln!("Error: Could not add shop item description: {}", error);
                let _ = tx.rollback().await;
                return Err(rocket::http::Status::InternalServerError);
            }
        };
    }
    match tx.commit().await {
        Ok(_) => Ok(Status::Ok),
        Err(error) => {
            eprintln!("Error: Could not commit transaction: {}", error);
            Err(rocket::http::Status::InternalServerError)
        }
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
    let compile_env = std::env::var("COMPILE_ENV").unwrap_or("prod".to_string());
    let rocket_instance = rocket::build().attach(Db::init()).mount(
        "/",
        routes![
            files,
            shop_solidjs,
            users,
            create_user,
            shop_items,
            create_shop_item,
            blogs,
            blog_contents,
            create_blog,
            shop_item_descs,
            create_shop_item_desc,
            create_shop_item_desc_many,
            shop_item_images,
            create_shop_item_image,
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
            tags_by_category
        ],
    );

    if compile_env == r#"prod"# {
        println!("Rocket is in prod mode. CORS is in default state.");
        rocket_instance
    } else {
        println!("Rocket is in Dev mode. CORS will allow prod-unsafe features.");
        let cors = CorsOptions::default()
            .allowed_origins(AllowedOrigins::all())
            .allowed_methods(
                vec![Method::Get, Method::Post, Method::Patch]
                    .into_iter()
                    .map(From::from)
                    .collect(),
            )
            .allow_credentials(true);
        rocket_instance.attach(cors.to_cors().unwrap())
    }
}
