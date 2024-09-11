use crate::Db;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

use super::project_item::ProjectItem;

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
    pub async fn add_or_get(&self, mut db: Connection<Db>) -> Result<Tag, sqlx::Error> {
        let result = sqlx::query_as!(
            Tag,
            "
                WITH newrow AS (
                    INSERT INTO tag (text) 
                    SELECT $1::VARCHAR
                    WHERE NOT EXISTS (SELECT * FROM tag WHERE text = $1)
                    RETURNING id, text
                )
                    SELECT id, COALESCE(text, '') AS \"text!\" FROM newrow
                    UNION
                    SELECT id, text FROM tag WHERE text = $1
            ",
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

    pub async fn get_all(mut db: Connection<Db>) -> Result<Vec<Tag>, sqlx::Error> {
        sqlx::query_as("SELECT id, text FROM tag")
            .fetch_all(&mut **db)
            .await
        // TODO: Add custom completion prints
    }

    pub async fn get_tags_by_project(
        mut db: Connection<Db>,
        project_item: &ProjectItem,
    ) -> Result<Vec<Tag>, sqlx::Error> {
        sqlx::query_as!(
            Tag,
            "
                SELECT tag.id, tag.text FROM tag 
                    INNER JOIN project_tech_tag ON tag.id=project_tech_tag.tag_id
                    WHERE project_tech_tag.project_id = $1
            ",
            project_item.id
        )
        .fetch_all(&mut **db)
        .await
        // TODO: Add custom completion prints
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
