use rocket::http::Status;
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_db_pools::Connection;

use crate::db::shop_item::{ShopImage, ShopItem, ShopItemDesc, ShopItemDescMany};
use crate::Db;
use sqlx::Either::{Left, Right};
use sqlx::Acquire;

#[get("/api/shopitems")]
pub async fn shop_items(db: Connection<Db>) -> Result<Json<Vec<ShopItem>>, Status> {
    match ShopItem::get_all(db).await {
        Ok(results) => Ok(Json(results)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/api/shopitem", data = "<shop_item>", format = "json")]
pub async fn create_shop_item(
    db: Connection<Db>,
    mut shop_item: Json<ShopItem>,
) -> Result<Created<Json<ShopItem>>, Status> {
    let shop_item_deser = ShopItem {
        id: None,
        iname: shop_item.iname.clone(),
        img_link: shop_item.img_link.clone(),
        price: shop_item.price,
    };
    let result = shop_item_deser.add(db).await;

    match result {
        Ok(query_result) => {
            if let Some(resulted_id) = query_result.id {
                shop_item.id = Some(resulted_id);
                Ok(Created::new("/").body(shop_item))
            } else {
                Err(Status::InternalServerError)
            }
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/api/shopitemimages/<id>")]
pub async fn shop_item_images(
    db: Connection<Db>,
    id: i32,
) -> Result<Json<Vec<ShopImage>>, Status> {
    match ShopImage::get_all_from_shop_item(db, id).await {
        Ok(results) => Ok(Json(results)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/api/shopitemimage", data = "<shop_item_image>", format = "json")]
pub async fn create_shop_item_image(
    db: Connection<Db>,
    shop_item_image: Json<ShopImage>,
) -> Result<Created<Json<ShopImage>>, Status> {
    let shop_item_desc_deser = ShopImage {
        id: None,
        shop_item_id: shop_item_image.shop_item_id,
        tooltip: shop_item_image.tooltip.clone(),
        img_link: shop_item_image.img_link.clone(),
    };
    let result = match shop_item_desc_deser.add(db).await {
        Ok(query_result) => query_result,
        Err(error) => match error {
            Left(sql_error) => {
                eprintln!("SQL Error occurred: {:?}", sql_error);
                return Err(Status::InternalServerError);
            }
            Right(_) => {
                eprintln!("Type not found for 'shop_item_id'");
                return Err(Status::BadRequest);
            }
        },
    };

    match result.id {
        Some(_) => Ok(Created::new("/").body(Json(result))),
        None => {
            eprintln!("Error: Row not found for the given shop item image.");
            Err(Status::NotFound)
        }
    }
}

#[get("/api/shopitemdescs/<id>")]
pub async fn shop_item_descs(
    db: Connection<Db>,
    id: i32,
) -> Result<Json<Vec<ShopItemDesc>>, Status> {
    match ShopItemDesc::get_all_from_shop_item(db, id).await {
        Ok(results) => Ok(Json(results)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/api/shopitemdesc", data = "<shop_item_desc>", format = "json")]
pub async fn create_shop_item_desc(
    db: Connection<Db>,
    shop_item_desc: Json<ShopItemDesc>,
) -> Result<Created<Json<ShopItemDesc>>, Status> {
    let shop_item_desc_deser = ShopItemDesc {
        id: None,
        shop_item_id: shop_item_desc.shop_item_id,
        content: shop_item_desc.content.clone(),
    };
    let result = match shop_item_desc_deser.add(db).await {
        Ok(query_result) => query_result,
        Err(error) => match error {
            Left(_) => {
                return Err(Status::InternalServerError);
            }
            Right(_) => {
                return Err(Status::BadRequest);
            }
        },
    };

    match result.id {
        Some(_) => Ok(Created::new("/").body(Json(result))),
        None => Err(Status::NotFound),
    }
}

#[post(
    "/api/shopitemdesc/many",
    data = "<shop_item_desc_many>",
    format = "json"
)]
pub async fn create_shop_item_desc_many(
    mut db: Connection<Db>,
    shop_item_desc_many: Json<ShopItemDescMany>,
) -> Result<Status, Status> {
    let mut tx = match (*db).begin().await {
        Ok(tx) => tx,
        Err(error) => {
            eprintln!("Error: Could not start transaction: {}", error);
            return Err(Status::InternalServerError);
        }
    };
    for content in &shop_item_desc_many.contents {
        match sqlx::query_as!(
            ShopItemDesc,
            "INSERT INTO shop_item_desc (shop_item_id, content) VALUES ($1, $2)",
            shop_item_desc_many.shop_item_id,
            content
        )
        .execute(&mut *tx)
        .await
        {
            Ok(_) => continue,
            Err(error) => {
                eprintln!("Error: Could not add shop item description: {}", error);
                let _ = tx.rollback().await;
                return Err(Status::InternalServerError);
            }
        };
    }
    match tx.commit().await {
        Ok(_) => Ok(Status::Ok),
        Err(error) => {
            eprintln!("Error: Could not commit transaction: {}", error);
            Err(Status::InternalServerError)
        }
    }
}
