use serde::{Deserialize, Serialize};

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
