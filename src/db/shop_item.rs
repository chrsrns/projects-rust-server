use postgres::Client;

pub struct ShopItem {
    pub id: Option<i32>,
    pub name: String,
    pub img_link: String,
    pub price: f32,
}

impl ShopItem {
    pub fn add(&self, client: &mut Client) -> Result<(), &str> {
        let result = if self.id.is_none() {
            client.execute(
                "INSERT INTO shop_item (id, name, img_link, price) VALUES ($1, $2, $3, $4)",
                &[&self.id, &self.name, &self.img_link, &self.price],
            )
        } else {
            client.execute(
                "INSERT INTO shop_item (name, img_link, price) VALUES ($1, $2, $3)",
                &[&self.name, &self.img_link, &self.price],
            )
        };

        match result {
            Ok(_) => {
                println!("Successfully added new shop item {}", &self.name);
                Ok(())
            }
            Err(_) => {
                // TODO: Use same text on print and an error result string
                println!("Error when creating new shop item with: [ {} ]", &self.name);
                Err("Error when creating new shop item")
            }
        }
    }

    pub fn get_by_id(client: &mut Client, id: i32) -> Result<ShopItem, &str> {
        let query = client.query(
            "SELECT id, name, img_link, price FROM shop_item WHERE id=$1",
            &[&id],
        );

        match query {
            Ok(rows) => {
                if !rows.is_empty() {
                    let row = &rows[0];

                    let id: i32 = row.get(0);
                    let name: String = row.get(1);
                    let img_link: String = row.get(2);
                    let price: f32 = row.get(3);
                    let shop_item = ShopItem {
                        id: Some(id),
                        name,
                        img_link,
                        price,
                    };
                    return Ok(shop_item);
                }
                Err("No shop item found.")
            }
            Err(_) => Err("Error occurred in querying the database"),
        }
    }
}
