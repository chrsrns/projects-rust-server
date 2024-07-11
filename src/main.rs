use rocket::fs::{FileServer, relative};

#[macro_use] extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/online-shopping-solidjs", FileServer::from(relative!("dist")))
}

