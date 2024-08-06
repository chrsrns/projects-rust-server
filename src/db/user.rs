use futures::stream::TryStreamExt;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

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
    pub async fn add(&self, mut db: Connection<Db>) -> Result<User, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO app_user (username, upassword, email) VALUES ($1, $2, $3 ) RETURNING id",
            &self.username,
            &self.upassword,
            &self.email
        )
        .fetch(&mut **db)
        .try_collect::<Vec<_>>()
        .await;

        match result {
            Ok(result) => {
                println!("Successfully added new user {}", &self.username);
                let id_returned = result.first().expect("returning result").id;
                Ok(User {
                    id: Some(id_returned),
                    username: self.username.clone(),
                    upassword: self.upassword.clone(),
                    email: self.email.clone(),
                })
            }
            Err(error) => {
                println!("Error when creating new user with: [ {} ]", &self.username);
                Err(error)
            }
        }
    }

    pub async fn get_all_users(mut db: Connection<Db>) -> Result<Vec<User>, sqlx::Error> {
        let results = sqlx::query_as!(User, "SELECT id, username, upassword, email FROM app_user")
            .fetch_all(&mut **db)
            .await;

        // TODO: Add custom completion prints
        match results {
            Ok(results_ok) => Ok(results_ok),
            Err(error) => Err(error),
        }
    }
}
