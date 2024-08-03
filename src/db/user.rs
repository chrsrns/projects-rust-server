use futures::stream::TryStreamExt;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;

use crate::Db;

#[derive(Serialize, Deserialize)]
pub struct User {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub username: String,
    pub upassword: String,
    pub email: String,
}

// TODO: Update/copy implementation from shop_items, which has simpler better handling.
impl User {
    pub async fn add(&self, mut db: Connection<Db>) -> Result<Vec<PgRow>, sqlx::Error> {
        let result = if self.id.is_none() {
            sqlx::query!(
                "INSERT INTO app_user (username, upassword, email) VALUES ($1, $2, $3 )",
                &self.username,
                &self.upassword,
                &self.email
            )
            .fetch(&mut **db)
            .try_collect::<Vec<_>>()
            .await
        } else {
            let id_unwrapped = &self.id.unwrap();
            sqlx::query!(
                "INSERT INTO app_user (id, username, upassword, email) VALUES ($1, $2, $3, $4)",
                &id_unwrapped,
                &self.username,
                &self.upassword,
                &self.email
            )
            .fetch(&mut **db)
            .try_collect::<Vec<_>>()
            .await
        };

        match result {
            Ok(result) => {
                println!("Successfully added new user {}", &self.username);
                Ok(result)
            }
            Err(error) => {
                println!("Error when creating new user with: [ {} ]", &self.username);
                Err(error)
            }
        }
    }

    pub async fn get_all_users(mut db: Connection<Db>) -> Result<Vec<User>, sqlx::Error> {
        let results = sqlx::query!("SELECT id, username, upassword, email FROM app_user")
            .fetch(&mut **db)
            .map_ok(|r| User {
                id: Some(r.id),
                username: r.username,
                upassword: r.upassword,
                email: r.email,
            })
            .try_collect::<Vec<_>>()
            .await;

        match results {
            Ok(results_ok) => {
                return Ok(results_ok);
            }
            Err(error) => return Err(error),
        }
    }
}
