use futures::stream::TryStreamExt;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

use crate::Db;

#[derive(Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "content_type", rename_all = "lowercase")]
pub enum ContentType {
    BigHeader,
    Header,
    SmallHeader,
    Body,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[sqlx(type_name = "blog_item")]
pub struct BlogItem {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub blog_title: String,
    pub header_img: String,
    #[sqlx(skip)]
    pub content: Vec<Content>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[sqlx(type_name = "content")]
pub struct Content {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub blog_id: i32,
    pub ctype: ContentType,
    pub content: String,
}
