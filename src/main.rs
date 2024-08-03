#[macro_use]
extern crate rocket;

use db::user::User;
use postgres::{Client, NoTls};
use rocket::fs::NamedFile;
use std::path::{Path, PathBuf};

mod db;

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
