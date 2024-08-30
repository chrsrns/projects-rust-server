#[macro_use]
extern crate rocket;

use db::blog_item::{BlogItem, Content};
use db::shop_item::{ShopImage, ShopItem, ShopItemDesc};
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

#[post("/api/user", data = "<user>")]
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

#[post("/api/shopitem", data = "<shop_item>")]
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

#[post("/api/shopitemimage", data = "<shop_item_image>")]
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

#[post("/api/shopitemdesc", data = "<shop_item_desc>")]
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

#[derive(Serialize, Deserialize)]
pub struct ShopItemDescMany {
    pub id: Option<i32>,
    pub shop_item_id: Option<i32>,
    pub contents: Vec<String>,
}
// TODO: Copy over the format parameter to other routes
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

#[post("/api/blog", data = "<blog_item>")]
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
    let resuilts = Content::get_all_from_blog(db, id).await?;
    Ok(Json(resuilts))
}

#[get("/api/users")]
async fn users(db: Connection<Db>) -> Result<Json<Vec<User>>> {
    let results = User::get_all_users(db).await?;
    Ok(Json(results))
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
    rocket::build()
        .attach(Db::init())
        .mount(
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
                create_shop_item_image
            ],
        )
        .attach(cors.to_cors().unwrap())
}
