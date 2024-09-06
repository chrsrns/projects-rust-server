use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[sqlx(type_name = "project_item")]
pub struct ProjectItem {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub title: String,
    pub thumbnail_img_link: String,
    pub desc: Vec<DescItem>,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
#[sqlx(type_name = "project_desc_item")]
pub struct DescItem {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub project_id: Option<i32>,
    pub content: String,
}
