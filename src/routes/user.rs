use rocket::http::Status;
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use crate::db::user::User;
use crate::Db;

#[get("/api/users")]
pub async fn users(db: Connection<Db>) -> Result<Json<Vec<User>>, rocket::http::Status> {
    match User::get_all_users(db).await {
        Ok(results) => Ok(Json(results)),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[post("/api/user", data = "<user>", format = "json")]
pub async fn create_user(
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