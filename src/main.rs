#[macro_use]
extern crate rocket;

use db::user::User;
use postgres::{Client, NoTls};
use rocket::fs::NamedFile;
use std::path::{Path, PathBuf};

mod db;

#[get("/assets/<file..>")]
async fn shop_solidjs(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("online-shopping-solidjs/assets/").join(file))
        .await
        .ok()
}

#[get("/<_..>", rank = 2)]
async fn files() -> Option<NamedFile> {
    NamedFile::open(Path::new("online-shopping-solidjs/index.html"))
        .await
        .ok()
}

#[launch]
fn rocket() -> _ {
    // TODO: Properly handle err case, instead of confidently unwrapping
    let mut client = Client::connect(
        "postgresql://postgres:postgres@localhost:5432/postgres",
        NoTls,
    )
    .unwrap();

    // TODO: Consider moving the function defs behind an encapsulated API
    db::init_database(&mut client);
    match User::get_all_users(&mut client) {
        Ok(results) => {
            for result in results {
                println!("Username: {}", result.username);
            }
        }
        Err(_) => {
            println!("Error when getting all users");
        }
    }
    rocket::build().mount("/online-shopping-solidjs", routes![files, shop_solidjs])
}
