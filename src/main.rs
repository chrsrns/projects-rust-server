#[macro_use] extern crate rocket;

use rocket::fs::NamedFile;
use std::path::{PathBuf, Path};

#[get("/assets/<file..>")]
async fn shop_solidjs(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("online-shopping-solidjs/assets/").join(file)).await.ok()
}

#[get("/<_..>", rank = 2)]
async fn files() -> Option<NamedFile> {
    NamedFile::open(Path::new("online-shopping-solidjs/index.html")).await.ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/online-shopping-solidjs", routes![files, shop_solidjs])
}

