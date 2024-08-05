use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ContentType {
    Header,
    Body,
}

#[derive(Serialize, Deserialize)]
pub struct BlogItem {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub blog_title: String,
    pub header_img: String,
    pub content: Vec<Content>,
}

#[derive(Serialize, Deserialize)]
pub struct Content {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub ctype: ContentType,
}
