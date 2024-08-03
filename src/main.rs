#[macro_use]
extern crate rocket;

use db::user::User;
use futures::stream::TryStreamExt;
use rocket::serde::json::Json;
use rocket::{fairing, Build};
use rocket::{fs::NamedFile, Rocket};
use rocket_db_pools::{Connection, Database};
use std::path::{Path, PathBuf};

mod db;

#[derive(Database)]
#[database("sqlx")]
struct Db(sqlx::PgPool);

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

#[launch]
async fn rocket() -> _ {

    rocket::build()
        .attach(Db::init())
        .mount("/", routes![files, shop_solidjs, users])
}
