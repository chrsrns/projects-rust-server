use futures::stream::TryStreamExt;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::Either::{self};

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

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[sqlx(type_name = "shop_image")]
pub struct ShopImage {
    pub id: Option<i32>,
    pub shop_item_id: Option<i32>,
    pub tooltip: String,
    pub img_link: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[sqlx(type_name = "shop_item_desc")]
pub struct ShopItemDesc {
    pub id: Option<i32>,
    pub shop_item_id: Option<i32>,
    pub content: String,
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

impl ShopImage {
    pub async fn add(&self, mut db: Connection<Db>) -> Result<ShopImage, Either<sqlx::Error, ()>> {
        match &self.shop_item_id {
            Some(shop_item_id) => {
                // TODO: Copy this implementation of query_as to the other insert functions
                let result = sqlx::query_as!(ShopImage,
                    "INSERT INTO shop_image (shop_item_id, tooltip, img_link) VALUES ($1, $2, $3) RETURNING id, shop_item_id, tooltip, img_link"
                    , shop_item_id, &self.tooltip, &self.img_link
                ).fetch_one(&mut **db).await;

                match result {
                    Ok(resulting_shop_image) => {
                        println!(
                            "Successfully added new shop item image {}",
                            resulting_shop_image.tooltip
                        );
                        Ok(resulting_shop_image)
                    }
                    Err(error) => {
                        println!(
                            "Error when creating new shop item image with tooltip: {}",
                            &self.tooltip
                        );
                        Err(Either::Left(error))
                    }
                }
            }
            None => Err(Either::Right(())),
        }
    }

    pub async fn get_all_from_shop_item(
        mut db: Connection<Db>,
        id: i32,
    ) -> Result<Vec<ShopImage>, sqlx::Error> {
        sqlx::query_as!(
            ShopImage,
            "SELECT id, shop_item_id, tooltip, img_link FROM shop_image WHERE shop_item_id=$1",
            id
        )
        .fetch_all(&mut **db)
        .await
    }
}

impl ShopItemDesc {
    pub async fn add(
        &self,
        mut db: Connection<Db>,
    ) -> Result<ShopItemDesc, Either<sqlx::Error, ()>> {
        match &self.shop_item_id {
            Some(shop_item_id) => {
                // TODO: Copy this implementation of query_as to the other insert functions
                let result = sqlx::query_as!(ShopItemDesc,
                    "INSERT INTO shop_item_desc (shop_item_id, content) VALUES ($1, $2) RETURNING id, shop_item_id, content"
                    , shop_item_id, &self.content
                ).fetch_one(&mut **db).await;

                match result {
                    Ok(resulting_shop_image) => {
                        println!("Successfully added new shop item desc");
                        Ok(resulting_shop_image)
                    }
                    Err(error) => {
                        println!("Error when creating new shop item");
                        Err(Either::Left(error))
                    }
                }
            }
            None => Err(Either::Right(())),
        }
    }

    pub async fn get_all_from_shop_item(
        mut db: Connection<Db>,
        id: i32,
    ) -> Result<Vec<ShopItemDesc>, sqlx::Error> {
        sqlx::query_as!(
            ShopItemDesc,
            "SELECT id, shop_item_id, content FROM shop_item_desc WHERE shop_item_id=$1",
            id
        )
        .fetch_all(&mut **db)
        .await
    }
}
