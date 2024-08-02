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
    let client_connection = Client::connect(
        "postgresql://postgres:postgres@localhost:5432/postgres",
        NoTls,
    );
    if client_connection.is_err() {
        // Just print an error message. The attempt to unwrap will terminate the program.
        println!("Failed to connect to PostgreSQL database. Termination imminent...");
    }
    let mut client = client_connection.unwrap();

    // TODO: Consider moving the function defs behind an encapsulated API
    match db::init_database(&mut client) {
        Ok(_) => println!("Successfully initialized database"),
        Err(error) => {
            println!("Failed to initialize database. Aborting... ");
            let unwrapped = error.code().unwrap().code();
            println!("Error reason: {}", unwrapped);
        }
    };
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
