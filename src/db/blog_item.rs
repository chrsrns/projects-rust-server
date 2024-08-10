use either::*;
use futures::stream::TryStreamExt;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, Postgres, Transaction};

use crate::Db;

#[derive(Serialize, Deserialize, Clone, sqlx::Type)]
#[serde(rename_all = "lowercase")]
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

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
#[sqlx(type_name = "content")]
pub struct Content {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub blog_id: Option<i32>,
    pub ctype: ContentType,
    pub content: String,
}

impl BlogItem {
    pub async fn add(&self, mut db: Connection<Db>) -> Result<BlogItem, sqlx::Error> {
        let mut tx = (*db).begin().await?;
        let result = sqlx::query!(
            "INSERT INTO blog_item (blog_title, header_img) VALUES ($1, $2) RETURNING id",
            &self.blog_title,
            &self.header_img,
        )
        .fetch(&mut *tx)
        .try_collect::<Vec<_>>()
        .await;

        let mut pushed_content = Vec::new();

        match result {
            Ok(result) => {
                println!("Successfully added new  {}", &self.blog_title);
                let id_returned = result.first().expect("returning result").id;

                for content in &self.content {
                    let content_copy = Content {
                        id: None,
                        blog_id: Some(id_returned),
                        ctype: content.ctype.clone(),
                        content: content.content.clone(),
                    };
                    let result = content_copy.add_tx(&mut tx).await;

                    match result {
                        Ok(resulting_content) => {
                            pushed_content.push(resulting_content);
                            continue;
                        }
                        Err(error) => match error {
                            Left(sql_error) => return Err(sql_error),
                            Right(_) => {
                                return Err(sqlx::Error::TypeNotFound {
                                    type_name: String::from("blog_id"),
                                })
                            }
                        },
                    };
                }

                tx.commit().await?;

                for content_item in &pushed_content {
                    println!("Inserted content {}", content_item.blog_id.unwrap_or(-1));
                }

                Ok(BlogItem {
                    id: Some(id_returned),
                    blog_title: self.blog_title.clone(),
                    header_img: self.header_img.clone(),
                    content: pushed_content,
                })
            }
            Err(error) => {
                println!(
                    "Error when creating new blog item with: [ {} ]",
                    &self.blog_title
                );
                Err(error)
            }
        }
    }

    pub async fn get_all(mut db: Connection<Db>) -> Result<Vec<BlogItem>, sqlx::Error> {
        sqlx::query_as("SELECT id, blog_title, header_img FROM blog_item")
            .fetch_all(&mut **db)
            .await
        // TODO: Add custom completion prints
    }

    pub async fn query_contents(
        &mut self,
        db: Connection<Db>,
    ) -> Result<(), Either<sqlx::Error, ()>> {
        match &self.id {
            None => Err(Right(())),
            Some(blog_id) => {
                let contents_result = Content::get_all_from_blog(db, *blog_id).await;
                match contents_result {
                    Ok(mut contents) => {
                        self.content.append(&mut contents);
                        Ok(())
                    }
                    Err(error) => Err(Left(error)),
                }
            }
        }
    }
}

impl Content {
    pub async fn add(&self, db: &mut Connection<Db>) -> Result<Content, Either<sqlx::Error, ()>> {
        match &self.blog_id {
            Some(blog_id) => {
                let result = sqlx::query!(
                    "INSERT INTO content (blog_id, ctype, content) VALUES ($1, $2, $3) RETURNING id",
                    blog_id,
                    &self.ctype as &ContentType,
                    &self.content,
                )
                    .fetch_one(&mut ***db)
                .await;

                match result {
                    Ok(record) => {
                        println!("Successfully added new content");
                        let id_returned = record.id;
                        Ok(Content {
                            id: Some(id_returned),
                            blog_id: self.blog_id,
                            ctype: self.ctype.clone(),
                            content: self.content.clone(),
                        })
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

    pub async fn add_tx<'a>(
        &self,
        db: &mut Transaction<'a, Postgres>,
    ) -> Result<Content, Either<sqlx::Error, ()>> {
        match &self.blog_id {
            Some(blog_id) => {
                let result = sqlx::query!(
                    "INSERT INTO content (blog_id, ctype, content) VALUES ($1, $2, $3) RETURNING id",
                    blog_id,
                    &self.ctype as &ContentType,
                    &self.content,
                )
                    .fetch_one(&mut **db)
                .await;

                match result {
                    Ok(record) => {
                        println!("Successfully added new content");
                        let id_returned = record.id;
                        Ok(Content {
                            id: Some(id_returned),
                            blog_id: self.blog_id,
                            ctype: self.ctype.clone(),
                            content: self.content.clone(),
                        })
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

    pub async fn get_all_from_blog(
        mut db: Connection<Db>,
        blog_id: i32,
    ) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"SELECT id, blog_id, ctype as "ctype: ContentType", content FROM content WHERE blog_id=$1"#,
            blog_id
        )
        .fetch_all(&mut **db)
        .await
        // TODO: Add custom completion prints
    }
}
