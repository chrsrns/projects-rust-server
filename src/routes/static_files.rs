use rocket::fs::NamedFile;
use std::path::{Path, PathBuf};

#[get("/online-shopping-solidjs/assets/<file..>")]
pub async fn solidjs_assets(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("online-shopping-solidjs/assets/").join(file))
        .await
        .ok()
}

#[get("/online-shopping-solidjs/<_..>", rank = 2)]
pub async fn solidjs_index() -> Option<NamedFile> {
    NamedFile::open(Path::new("online-shopping-solidjs/index.html"))
        .await
        .ok()
}
