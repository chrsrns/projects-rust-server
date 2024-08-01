use postgres::{Client, Error};

pub struct User {
    pub id: Option<i32>,
    pub username: String,
    pub password: String,
    pub email: String,
}

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
        let result = client.execute(
            "INSERT INTO app_user (username, password, email) VALUES ($1, $2, $3)",
            &[&self.username, &self.password, &self.email],
        );

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
                password,
                email,
            };
            results.push(user);
        }

        Ok(results)
    }
}
