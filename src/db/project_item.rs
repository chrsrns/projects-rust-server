use either::{Either, Left, Right};
use futures::stream::TryStreamExt;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, Postgres, Transaction};

use super::tag::Tag;
use crate::Db;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[sqlx(type_name = "project_item")]
pub struct ProjectItem {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub title: String,
    pub thumbnail_img_link: String,
    #[sqlx(skip)]
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

impl ProjectItem {
    pub async fn add(&self, mut db: Connection<Db>) -> Result<ProjectItem, sqlx::Error> {
        let mut tx = (*db).begin().await?;
        let result = sqlx::query!(
            "INSERT INTO project_item (title, thumbnail_img_link) VALUES ($1, $2) RETURNING id",
            &self.title,
            &self.thumbnail_img_link,
        )
        .fetch(&mut *tx)
        .try_collect::<Vec<_>>()
        .await;

        let mut pushed_desc = Vec::new();

        match result {
            Ok(result) => {
                println!("Successfully added new  {}", &self.title);
                let id_returned = result.first().expect("returning result").id;

                for content in &self.desc {
                    let content_copy = DescItem {
                        id: None,
                        project_id: content.project_id,

                        content: content.content.clone(),
                    };
                    let result = content_copy.add_tx(&mut tx).await;

                    match result {
                        Ok(resulting_content) => {
                            pushed_desc.push(resulting_content);
                            continue;
                        }
                        Err(error) => match error {
                            Left(sql_error) => return Err(sql_error),
                            Right(_) => {
                                return Err(sqlx::Error::TypeNotFound {
                                    type_name: String::from("project_id"),
                                })
                            }
                        },
                    };
                }

                tx.commit().await?;

                for content_item in &pushed_desc {
                    println!("Inserted content {}", content_item.project_id.unwrap_or(-1));
                }

                Ok(ProjectItem {
                    id: Some(id_returned),
                    title: self.title.clone(),
                    thumbnail_img_link: self.thumbnail_img_link.clone(),
                    desc: pushed_desc,
                })
            }
            Err(error) => {
                println!(
                    "Error when creating new blog item with: [ {} ]",
                    &self.title
                );
                Err(error)
            }
        }
    }

    pub async fn add_tag(
        &self,
        mut db: Connection<Db>,
        tags: Vec<&Tag>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = (*db).begin().await?;

        for tag in tags {
            let result = sqlx::query!(
                "INSERT INTO project_tech_tag (project_id, tag_id) VALUES ($1, $2) RETURNING id",
                &self.id.unwrap_or(-1),
                tag.id
            )
            .fetch(&mut *tx)
            .try_collect::<Vec<_>>()
            .await;

            result?;
        }
        Ok(())
    }

    pub async fn get_projects_by_tab(
        mut db: Connection<Db>,
        tag: &Tag,
    ) -> Result<Vec<ProjectItem>, sqlx::Error> {
        sqlx::query_as(
            "
                SELECT project_item.id, project_item.thumbnail_img_link FROM project_item 
                    INNER JOIN project_tech_tag ON project_item.id=project_tech_tag.project_id
                    WHERE project_tech_tag.tag_id = $1
            ",
        )
        .bind(tag.id)
        .fetch_all(&mut **db)
        .await
        // TODO: Add custom completion prints
    }
}

impl DescItem {
    pub async fn add(&self, db: &mut Connection<Db>) -> Result<DescItem, Either<sqlx::Error, ()>> {
        match &self.project_id {
            Some(project_id) => {
                let result = sqlx::query_as!(DescItem,
                    "INSERT INTO project_desc_item (project_id, content) VALUES ($1, $2) RETURNING id, project_id, content",
                    project_id,
                    &self.content,
                )
                    .fetch_one(&mut ***db)
                .await;

                match result {
                    Ok(record) => {
                        println!("Successfully added new project description");
                        Ok(record)
                    }
                    Err(error) => {
                        println!("Error when creating new project description");
                        Err(Left(error))
                    }
                }
            }
            None => Err(Right(())),
        }
    }

    pub async fn add_tx<'a>(
        &self,
        db: &mut Transaction<'a, Postgres>,
    ) -> Result<DescItem, Either<sqlx::Error, ()>> {
        match &self.project_id {
            Some(project_id) => {
                let result = sqlx::query_as!(DescItem,
                    "INSERT INTO project_desc_item (project_id, content) VALUES ($1, $2) RETURNING id, project_id, content",
                    project_id,
                    &self.content,
                )
                    .fetch_one(&mut **db)
                .await;

                match result {
                    Ok(record) => {
                        println!("Successfully added new content");
                        Ok(record)
                    }
                    Err(error) => {
                        println!("Error when creating new content");
                        Err(Left(error))
                    }
                }
            }
            None => Err(Right(())),
        }
    }
}
