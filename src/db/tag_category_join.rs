use serde::{Deserialize, Serialize};

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
