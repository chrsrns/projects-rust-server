//! Main application entry point and server configuration
//! 
//! This module sets up the Rocket web server, configures CORS,
//! initializes the database connection, and mounts all route handlers.

#[macro_use]
extern crate rocket;

use rocket::http::Method;
use rocket::{fairing, Build};
use rocket::Rocket;
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_db_pools::Database;

mod db;
mod routes;
mod api;

/// Database connection pool wrapper for PostgreSQL
/// 
/// This struct represents the connection to our PostgreSQL database
/// using SQLx as the database driver.
#[derive(Database)]
#[database("sqlx")]
pub struct Db(sqlx::PgPool);

/// Initializes the database and runs migrations
/// 
/// # Arguments
/// * `rocket` - The Rocket instance to attach the database to
/// 
/// # Returns
/// * `fairing::Result` - Success if database initialization and migrations succeed
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

/// Configures and launches the Rocket web server
/// 
/// This function:
/// - Sets up CORS configuration
/// - Initializes the database connection
/// - Mounts all route handlers
/// - Launches the web server
/// 
/// # Returns
/// * The configured Rocket instance
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
                routes::user::users,
                routes::user::create_user,
            ],
        )
}
