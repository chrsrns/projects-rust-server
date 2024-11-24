use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_db_pools::Connection;

use crate::db::shop_item::{ShopImage, ShopItem, ShopItemDesc, ShopItemDescMany};
use crate::Db;
use crate::api::{ApiResponse, ApiResult, ApiError};
use sqlx::Either::{Left, Right};
use sqlx::Acquire;

#[get("/api/shopitems")]
pub async fn shop_items(db: Connection<Db>) -> ApiResult<Vec<ShopItem>> {
    match ShopItem::get_all(db).await {
        Ok(results) => Ok(ApiResponse::success(results)),
        Err(_) => Err(ApiError::new(
            "Failed to fetch shop items",
            Status::InternalServerError
        )),
    }
}

#[post("/api/shopitem", data = "<shop_item>", format = "json")]
pub async fn create_shop_item(
    db: Connection<Db>,
    shop_item: Json<ShopItem>,
) -> ApiResult<ShopItem> {
    let shop_item_deser = ShopItem {
        id: None,
        iname: shop_item.iname.clone(),
        img_link: shop_item.img_link.clone(),
        price: shop_item.price,
    };
    
    match shop_item_deser.add(db).await {
        Ok(query_result) => {
            if query_result.id.is_some() {
                Ok(ApiResponse::success(query_result))
            } else {
                Err(ApiError::new(
                    "Failed to create shop item: No ID returned",
                    Status::InternalServerError
                ))
            }
        }
        Err(_) => Err(ApiError::new(
            "Failed to create shop item",
            Status::InternalServerError
        )),
    }
}

#[get("/api/shopitemimages/<id>")]
pub async fn shop_item_images(
    db: Connection<Db>,
    id: i32,
) -> ApiResult<Vec<ShopImage>> {
    match ShopImage::get_all_from_shop_item(db, id).await {
        Ok(results) => Ok(ApiResponse::success(results)),
        Err(_) => Err(ApiError::new(
            "Failed to fetch shop item images",
            Status::InternalServerError
        )),
    }
}

#[post("/api/shopitemimage", data = "<shop_item_image>", format = "json")]
pub async fn create_shop_item_image(
    db: Connection<Db>,
    shop_item_image: Json<ShopImage>,
) -> ApiResult<ShopImage> {
    let shop_item_desc_deser = ShopImage {
        id: None,
        shop_item_id: shop_item_image.shop_item_id,
        tooltip: shop_item_image.tooltip.clone(),
        img_link: shop_item_image.img_link.clone(),
    };
    
    let result = match shop_item_desc_deser.add(db).await {
        Ok(query_result) => query_result,
        Err(error) => match error {
            Left(_) => {
                return Err(ApiError::new(
                    "Failed to create shop item image",
                    Status::InternalServerError
                ));
            }
            Right(_) => {
                return Err(ApiError::new(
                    "Invalid shop_item_id",
                    Status::BadRequest
                ));
            }
        },
    };

    match result.id {
        Some(_) => Ok(ApiResponse::success(result)),
        None => Err(ApiError::new(
            "Failed to create shop item image: No ID returned",
            Status::NotFound
        )),
    }
}

#[get("/api/shopitemdescs/<id>")]
pub async fn shop_item_descs(
    db: Connection<Db>,
    id: i32,
) -> ApiResult<Vec<ShopItemDesc>> {
    match ShopItemDesc::get_all_from_shop_item(db, id).await {
        Ok(results) => Ok(ApiResponse::success(results)),
        Err(_) => Err(ApiError::new(
            "Failed to fetch shop item descriptions",
            Status::InternalServerError
        )),
    }
}

#[post("/api/shopitemdesc", data = "<shop_item_desc>", format = "json")]
pub async fn create_shop_item_desc(
    db: Connection<Db>,
    shop_item_desc: Json<ShopItemDesc>,
) -> ApiResult<ShopItemDesc> {
    let shop_item_desc_deser = ShopItemDesc {
        id: None,
        shop_item_id: shop_item_desc.shop_item_id,
        content: shop_item_desc.content.clone(),
    };
    
    let result = match shop_item_desc_deser.add(db).await {
        Ok(query_result) => query_result,
        Err(error) => match error {
            Left(_) => {
                return Err(ApiError::new(
                    "Failed to create shop item description",
                    Status::InternalServerError
                ));
            }
            Right(_) => {
                return Err(ApiError::new(
                    "Invalid shop_item_id",
                    Status::BadRequest
                ));
            }
        },
    };

    match result.id {
        Some(_) => Ok(ApiResponse::success(result)),
        None => Err(ApiError::new(
            "Failed to create shop item description: No ID returned",
            Status::NotFound
        )),
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
) -> ApiResult<()> {
    let mut tx = match (*db).begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return Err(ApiError::new(
                "Failed to start transaction",
                Status::InternalServerError
            ));
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
            Err(_) => {
                let _ = tx.rollback().await;
                return Err(ApiError::new(
                    "Failed to create shop item descriptions",
                    Status::InternalServerError
                ));
            }
        };
    }

    match tx.commit().await {
        Ok(_) => Ok(ApiResponse::success(())),
        Err(_) => Err(ApiError::new(
            "Failed to commit transaction",
            Status::InternalServerError
        )),
    }
}
