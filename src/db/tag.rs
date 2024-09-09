use crate::Db;
use rocket_db_pools::Connection;
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

impl Tag {
    pub async fn add(&self, mut db: Connection<Db>) -> Result<Tag, sqlx::Error> {
        let result = sqlx::query_as!(
            Tag,
            "INSERT INTO tag (text) VALUES ($1) RETURNING id, text",
            &self.text,
        )
        .fetch_one(&mut **db)
        .await;

        match result {
            Ok(result) => {
                println!("Successfully added new  {}", &self.text);

                Ok(result)
            }
            Err(error) => {
                println!("Error when creating new blog item with: [ {} ]", &self.text);
                Err(error)
            }
        }
    }
}

impl ProjectToTechTag {
    pub async fn add(
        mut db: Connection<Db>,
        project_id: &i32,
        tag_id: &i32,
    ) -> Result<ProjectToTechTag, sqlx::Error> {
        let result = sqlx::query_as!(
            ProjectToTechTag,
            "INSERT INTO project_tech_tag (project_id, tag_id) VALUES ($1, $2) RETURNING id, project_id, tag_id",
&project_id, &tag_id
            ,
        )
        .fetch_one(&mut **db)
        .await;

        match result {
            Ok(result) => {
                println!("Successfully added new  {}", result.id.unwrap_or(-1));

                Ok(result)
            }
            Err(error) => {
                println!(
                    "Error when joining project {} and tag {}",
                    &project_id, &tag_id
                );
                Err(error)
            }
        }
    }
}
