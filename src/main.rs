#[macro_use]
extern crate rocket;

use db::blog_item::{BlogItem, Content};
use db::project_item::{DescItem, ProjectItem};
use db::shop_item::{ShopImage, ShopItem, ShopItemDesc, ShopItemDescMany};
use db::tag::Tag;
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

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

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
async fn create_user(db: Connection<Db>, mut user: Json<User>) -> Result<Created<Json<User>>> {
    let user_deser = User {
        id: None,
        username: user.username.clone(),
        upassword: user.upassword.clone(),
        email: user.email.clone(),
    };
    let result = user_deser.add(db).await?;

    match result.id {
        Some(resulted_id) => {
            user.id = Some(resulted_id);
            Ok(Created::new("/").body(user))
        }

        None => {
            // TODO: Improve error handling
            panic!("This shouldn't have happened, but it did");
        }
    }
}

#[get("/api/shopitems")]
async fn shop_items(db: Connection<Db>) -> Result<Json<Vec<ShopItem>>> {
    let results = ShopItem::get_all(db).await?;
    Ok(Json(results))
}

#[post("/api/shopitem", data = "<shop_item>", format = "json")]
async fn create_shop_item(
    db: Connection<Db>,
    mut shop_item: Json<ShopItem>,
) -> Result<Created<Json<ShopItem>>> {
    let shop_item_deser = ShopItem {
        id: None,
        iname: shop_item.iname.clone(),
        img_link: shop_item.img_link.clone(),
        price: shop_item.price,
    };
    let result = shop_item_deser.add(db).await?;

    match result.id {
        Some(resulted_id) => {
            shop_item.id = Some(resulted_id);
            Ok(Created::new("/").body(shop_item))
        }

        None => {
            // TODO: Improve error handling
            panic!("This shouldn't have happened, but it did");
        }
    }
}

#[get("/api/shopitemimages/<id>")]
async fn shop_item_images(db: Connection<Db>, id: i32) -> Result<Json<Vec<ShopImage>>> {
    Ok(Json(ShopImage::get_all_from_shop_item(db, id).await?))
}

#[post("/api/shopitemimage", data = "<shop_item_image>", format = "json")]
async fn create_shop_item_image(
    db: Connection<Db>,
    shop_item_image: Json<ShopImage>,
) -> Result<Created<Json<ShopImage>>> {
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
                return Err(rocket::response::Debug(sql_error));
            }
            Right(_) => {
                return Err(rocket::response::Debug(sqlx::Error::TypeNotFound {
                    type_name: String::from("shop_item_id"),
                }));
            }
        },
    };

    match result.id {
        Some(_) => Ok(Created::new("/").body(Json(result))),
        None => Err(rocket::response::Debug(sqlx::Error::RowNotFound)),
    }
}

#[get("/api/shopitemdescs/<id>")]
async fn shop_item_descs(db: Connection<Db>, id: i32) -> Result<Json<Vec<ShopItemDesc>>> {
    Ok(Json(ShopItemDesc::get_all_from_shop_item(db, id).await?))
}

#[post("/api/shopitemdesc", data = "<shop_item_desc>", format = "json")]
async fn create_shop_item_desc(
    db: Connection<Db>,
    shop_item_desc: Json<ShopItemDesc>,
) -> Result<Created<Json<ShopItemDesc>>> {
    let shop_item_desc_deser = ShopItemDesc {
        id: None,
        shop_item_id: shop_item_desc.shop_item_id,
        content: shop_item_desc.content.clone(),
    };
    let result = match shop_item_desc_deser.add(db).await {
        Ok(query_result) => query_result,
        Err(error) => match error {
            Left(sql_error) => {
                return Err(rocket::response::Debug(sql_error));
            }
            Right(_) => {
                return Err(rocket::response::Debug(sqlx::Error::TypeNotFound {
                    type_name: String::from("shop_item_id"),
                }));
            }
        },
    };

    match result.id {
        Some(_) => Ok(Created::new("/").body(Json(result))),
        None => Err(rocket::response::Debug(sqlx::Error::RowNotFound)),
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
) -> Result<Status> {
    let mut tx = (*db).begin().await?;
    for content in &shop_item_desc_many.contents {
        sqlx::query_as!(
            ShopItemDesc,
            "INSERT INTO shop_item_desc (shop_item_id, content) VALUES ($1, $2)",
            shop_item_desc_many.shop_item_id,
            content
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(Status::Ok)
}

#[get("/api/blogs")]
async fn blogs(db: Connection<Db>) -> Result<Json<Vec<BlogItem>>> {
    let results = BlogItem::get_all(db).await?;
    Ok(Json(results))
}

#[post("/api/blog", data = "<blog_item>", format = "json")]
async fn create_blog(
    db: Connection<Db>,
    blog_item: Json<BlogItem>,
) -> Result<Created<Json<BlogItem>>> {
    let blog_item_deser = BlogItem {
        id: None,
        blog_title: blog_item.blog_title.clone(),
        header_img: blog_item.header_img.clone(),
        content: blog_item.content.clone(),
    };
    let result = blog_item_deser.add(db).await?;

    Ok(Created::new("/").body(Json(result)))
}

#[get("/api/blog-content/<id>")]
async fn blog_contents(db: Connection<Db>, id: i32) -> Result<Json<Vec<Content>>> {
    let results = Content::get_all_from_blog(db, id).await?;
    Ok(Json(results))
}

#[get("/api/projects")]
async fn projects(db: Connection<Db>) -> Result<Json<Vec<ProjectItem>>> {
    let results = ProjectItem::get_all(db).await?;
    Ok(Json(results))
}

#[get("/api/projects-by-tag/<tag_id>")]
async fn projects_by_tag(db: Connection<Db>, tag_id: i32) -> Result<Json<Vec<ProjectItem>>> {
    let results = ProjectItem::get_projects_by_tag(db, tag_id).await?;
    Ok(Json(results))
}

#[post("/api/project", data = "<project_item>", format = "json")]
async fn create_project_item(
    db: Connection<Db>,
    project_item: Json<ProjectItem>,
) -> Result<Created<Json<ProjectItem>>> {
    let project_item_deser = ProjectItem {
        id: None,
        title: project_item.title.clone(),
        thumbnail_img_link: project_item.thumbnail_img_link.clone(),
        desc: project_item.desc.clone(),
    };
    let result = project_item_deser.add(db).await?;

    match result.id {
        Some(_) => Ok(Created::new("/").body(Json(result))),
        None => {
            // TODO: Improve error handling
            panic!("This shouldn't have happened, but it did");
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
async fn project_descs(db: Connection<Db>, id: i32) -> Result<Json<Vec<DescItem>>> {
    let resuilts = DescItem::get_all_from_project(db, id).await?;
    Ok(Json(resuilts))
}

#[post("/api/project_desc", data = "<project_desc>", format = "json")]
async fn create_project_desc(
    db: Connection<Db>,
    project_desc: Json<DescItem>,
) -> Result<Created<Json<DescItem>>> {
    let project_desc_deser = DescItem {
        id: None,
        project_id: project_desc.project_id,
        content: project_desc.content.clone(),
    };
    let result = match project_desc_deser.add(db).await {
        Ok(query_result) => query_result,
        Err(error) => match error {
            Left(sql_error) => {
                return Err(rocket::response::Debug(sql_error));
            }
            Right(_) => {
                return Err(rocket::response::Debug(sqlx::Error::TypeNotFound {
                    type_name: String::from("project_id"),
                }));
            }
        },
    };
    match result.id {
        Some(_) => Ok(Created::new("/").body(Json(result))),
        None => Err(rocket::response::Debug(sqlx::Error::RowNotFound)),
    }
}

#[post("/api/project_desc", data = "<project_descs>", format = "json")]
async fn create_project_desc_many(
    mut db: Connection<Db>,
    project_descs: Json<Vec<DescItem>>,
) -> Result<Status> {
    let mut tx = (*db).begin().await?;

    // TODO: Janky Error handling. Rewrite to be similar to many function somewhere above
    for project_desc in project_descs.iter() {
        let result = project_desc.add_tx(&mut tx).await;
        match result {
            Err(error) => {
                if error.is_left() {
                    return Err(rocket::response::Debug(error.unwrap_left()));
                } else {
                    return Ok(Status::UnprocessableEntity);
                }
            }
            _ => continue,
        }
    }
    tx.commit().await?;
    Ok(Status::Ok)
}

#[get("/api/tags")]
async fn tags(db: Connection<Db>) -> Result<Json<Vec<Tag>>> {
    let results = Tag::get_all(db).await?;
    Ok(Json(results))
}

#[post("/api/tag", data = "<tag>", format = "json")]
async fn create_tag(db: Connection<Db>, tag: Json<Tag>) -> Result<Created<Json<Tag>>> {
    let tag_deser = Tag {
        id: None,
        text: tag.text.clone(),
    };
    let result = tag_deser.add_or_get(db).await?;
    Ok(Created::new("/").body(Json(result)))
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

#[get("/api/users")]
async fn users(db: Connection<Db>) -> Result<Json<Vec<User>>> {
    let results = User::get_all_users(db).await?;
    Ok(Json(results))
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
