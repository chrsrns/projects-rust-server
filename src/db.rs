use postgres::{Client, Error};
use user::User;

pub mod user;

pub fn init_database(client: &mut Client) -> Result<(), Error> {
    client.batch_execute(
        "
        CREATE TABLE IF NOT EXISTS app_user (
            id              SERIAL PRIMARY KEY,
            username        VARCHAR UNIQUE NOT NULL,
            password        VARCHAR NOT NULL,
            email           VARCHAR UNIQUE NOT NULL
            )
    ",
    )?;

    // TODO: Remove after the db code is properly implemented.
    let new_user = User {
        id: None,
        username: String::from("aranas"),
        password: String::from("aranas"),
        email: String::from("aranaschristianlouise@gmail.com"),
    };
    new_user.add(client);

    Ok(())
}
