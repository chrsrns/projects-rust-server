#[macro_use]
extern crate rocket;

use db::user::User;
use rocket::http::{Method, Status};
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::{fairing, Build};
use rocket::Rocket;
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_db_pools::{Connection, Database};

mod db;
mod routes;

#[derive(Database)]
#[database("sqlx")]
pub struct Db(sqlx::PgPool);

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

#[post("/api/user", data = "<user>", format = "json")]
async fn create_user(
    db: Connection<Db>,
    user: Json<User>,
) -> Result<Created<Json<User>>, Status> {
    let user_deser = User {
        id: None,
        username: user.username.clone(),
        upassword: user.upassword.clone(),
        email: user.email.clone(),
    };

    match user_deser.add(db).await {
        Ok(result) => {
            let resulted_id = result.id.expect("This shouldn't have happened, but it did");
            let user = Json(User {
                id: Some(resulted_id),
                username: user.username.clone(),
                upassword: user.upassword.clone(),
                email: user.email.clone(),
            });
            Ok(Created::new("/").body(user))
        }
        Err(_) => Err(Status::InternalServerError),
    }
}


#[get("/api/users")]
async fn users(db: Connection<Db>) -> Result<Json<Vec<User>>, rocket::http::Status> {
    match User::get_all_users(db).await {
        Ok(results) => Ok(Json(results)),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
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

    let rocket = rocket::build();
    let rocket = match init_database(rocket).await {
        Ok(rocket) => rocket,
        Err(r) => r,
    };

    rocket
        .attach(cors.to_cors().unwrap())
        .attach(Db::init())
        .mount(
            "/",
            routes![
                routes::static_files::solidjs_assets,
                routes::static_files::solidjs_index,
                routes::shop::shop_items,
                routes::shop::create_shop_item,
                routes::shop::shop_item_images,
                routes::shop::create_shop_item_image,
                routes::shop::shop_item_descs,
                routes::shop::create_shop_item_desc,
                routes::shop::create_shop_item_desc_many,
                routes::blog::blogs,
                routes::blog::blog_contents,
                routes::blog::create_blog,
                routes::project::projects,
                routes::project::projects_by_tag,
                routes::project::add_tags_to_project,
                routes::project::project_descs,
                routes::project::create_project_item,
                routes::project::create_project_desc,
                routes::project::create_project_desc_many,
                routes::tag::tags,
                routes::tag::tag_category,
                routes::tag::tag_project,
                routes::tag::create_tag,
                routes::tag::tags_by_project,
                routes::tag::tags_by_category,
                users,
                create_user,
            ],
        )
}
