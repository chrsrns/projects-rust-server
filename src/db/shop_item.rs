use futures::stream::TryStreamExt;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

use crate::Db;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[sqlx(type_name = "shop_item")]
pub struct ShopItem {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub iname: String,
    pub img_link: String,
    pub price: f32,
}

impl ShopItem {
    pub async fn add(&self, mut db: Connection<Db>) -> Result<ShopItem, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO shop_item (iname, img_link, price) VALUES ($1, $2, $3) RETURNING id",
            &self.iname,
            &self.img_link,
            &self.price
        )
        .fetch(&mut **db)
        .try_collect::<Vec<_>>()
        .await;

        match result {
            Ok(result) => {
                println!("Successfully added new  {}", &self.iname);
                let id_returned = result.first().expect("returning result").id;
                Ok(ShopItem {
                    id: Some(id_returned),
                    iname: self.iname.clone(),
                    img_link: self.img_link.clone(),
                    price: self.price,
                })
            }
            Err(error) => {
                println!(
                    "Error when creating new shop item with: [ {} ]",
                    &self.iname
                );
                Err(error)
            }
        }
    }

    pub async fn get_by_id(mut db: Connection<Db>, id: i32) -> Result<ShopItem, sqlx::Error> {
        sqlx::query_as!(
            ShopItem,
            "SELECT id, iname, img_link, price FROM shop_item WHERE id=$1",
            id,
        )
        .fetch_one(&mut **db)
        .await
        // TODO: Add custom completion prints
    }

    pub async fn get_all(mut db: Connection<Db>) -> Result<Vec<ShopItem>, sqlx::Error> {
        sqlx::query_as!(ShopItem, "SELECT id, iname, img_link, price FROM shop_item",)
            .fetch_all(&mut **db)
            .await
        // TODO: Add custom completion prints
    }
}
