#[macro_use]
extern crate rocket;

use db_models::User;
use postgres::{Client, Error, NoTls};
use rocket::fs::NamedFile;
use std::path::{Path, PathBuf};

mod db_models;

fn init_database(client: &mut Client) -> Result<(), Error> {
    client.batch_execute(
        "
        CREATE TABLE IF NOT EXISTS app_user (
            id              SERIAL PRIMARY KEY,
            username        VARCHAR UNIQUE NOT NULL,
            password        VARCHAR NOT NULL,
            email           VARCHAR UNIQUE NOT NULL
            )
    ",
    )?;

    // TODO: Remove after the db code is properly implemented.
    add_user(client, "user2", "mypass", "user2@test.com")?;

    Ok(())
}

fn add_user(client: &mut Client, username: &str, password: &str, email: &str) -> Result<(), Error> {
    let result = client.execute(
        "INSERT INTO app_user (username, password, email) VALUES ($1, $2, $3)",
        &[&username, &password, &email],
    );

    match result {
        Ok(_) => {
            println!("Successfully added new user {}", username);
            Ok(())
        }
        Err(error) => {
            println!("Error when creating new user with: [ {} ]", username);
            Err(error)
        }
    }
}

fn get_all_users(client: &mut Client) -> Result<Vec<User>, Error> {
    let mut results = Vec::new();

    for row in client.query("SELECT id, username, password, email FROM app_user", &[])? {
        let id: i32 = row.get(0);
        let username: String = row.get(1);
        let password: String = row.get(2);
        let email: String = row.get(3);
        let user = User {
            id,
            username,
            password,
            email,
        };
        results.push(user);
    }

    Ok(results)
}

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
    init_database(&mut client);
    match get_all_users(&mut client) {
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
