use postgres::{Client, Error};
use serde::{Deserialize, Serialize};

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
    pub fn add_user(
        client: &mut Client,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<(), Error> {
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

    pub fn add(&self, client: &mut Client) -> Result<(), Error> {
        let result = if self.id.is_none() {
            client.execute(
                "INSERT INTO app_user (id, username, password, email) VALUES ($1, $2, $3, $4)",
                &[&self.id, &self.username, &self.upassword, &self.email],
            )
        } else {
            client.execute(
                "INSERT INTO app_user (username, password, email) VALUES ($1, $2, $3)",
                &[&self.username, &self.upassword, &self.email],
            )
        };

        match result {
            Ok(_) => {
                println!("Successfully added new user {}", &self.username);
                Ok(())
            }
            Err(error) => {
                println!("Error when creating new user with: [ {} ]", &self.username);
                Err(error)
            }
        }
    }

    pub fn get_all_users(client: &mut Client) -> Result<Vec<User>, Error> {
        let mut results = Vec::new();

        for row in client.query("SELECT id, username, password, email FROM app_user", &[])? {
            let id: i32 = row.get(0);
            let username: String = row.get(1);
            let password: String = row.get(2);
            let email: String = row.get(3);
            let user = User {
                id: Some(id),
                username,
                upassword: password,
                email,
            };
            results.push(user);
        }

        Ok(results)
    }
}
