#[macro_use]
extern crate rocket;

use db::shop_item::ShopItem;
use db::user::User;
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::{fairing, Build};
use rocket::{fs::NamedFile, Rocket};
use rocket_db_pools::{Connection, Database};
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
    // NOTE: sqlx#2543, sqlx#1648 mean we can't use the pithier `fetch_one()`.
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
    // NOTE: sqlx#2543, sqlx#1648 mean we can't use the pithier `fetch_one()`.
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

#[get("/api/users")]
async fn users(db: Connection<Db>) -> Result<Json<Vec<User>>> {
    let results = User::get_all_users(db).await?;
    Ok(Json(results))
}

#[launch]
async fn rocket() -> _ {
    rocket::build().attach(Db::init()).mount(
        "/",
        routes![
            files,
            shop_solidjs,
            users,
            create_user,
            shop_items,
            create_shop_item
        ],
    )
}
