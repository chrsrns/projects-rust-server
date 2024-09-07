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

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
#[sqlx(type_name = "tag")]
pub struct Tag {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
#[sqlx(type_name = "project_tech_tag")]
pub struct ProjectToTechTag {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub project_id: i32,
    pub tag_id: i32,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
#[sqlx(type_name = "tag_category_join")]
pub struct TagCategoryJoin {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub tag_id: i32,
    pub tag_category: TagCategory,
}

#[derive(Serialize, Deserialize, Clone, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "tag_category", rename_all = "lowercase")]
pub enum TagCategory {
    Language,
    Framework,
    Database,
}
