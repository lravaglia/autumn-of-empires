use async_trait::async_trait;
use color_eyre::Result;
use sqlx::prelude::*;

#[async_trait]
pub trait GetAll: Sized {
    async fn all(connection: &sqlx::SqlitePool) -> Result<Vec<Self>>;
}

#[derive(Debug, Default, PartialEq, Eq, FromRow)]
pub struct Ship {
    pub id: String,
    pub name: String,
    pub fleet: String,
    pub integrity: i64,
}

#[async_trait]
impl GetAll for Ship {
    async fn all(connection: &sqlx::SqlitePool) -> Result<Vec<Self>> {
        let mut conn = connection.acquire().await?;
        let r = sqlx::query_as!(Self, "select * from ships")
            .fetch_all(&mut conn)
            .await?;
        Ok(r)
    }
}

#[derive(Debug, Default, PartialEq, Eq, FromRow)]
pub struct Attack {
    pub id: i64,
    pub target: String,
}

#[async_trait]
impl GetAll for Attack {
    async fn all(connection: &sqlx::SqlitePool) -> Result<Vec<Self>> {
        let mut conn = connection.acquire().await?;
        let r = sqlx::query_as!(Self, "select * from attacks")
            .fetch_all(&mut conn)
            .await?;
        Ok(r)
    }
}

#[derive(Debug, Default, PartialEq, Eq, FromRow)]
pub struct Fleet {
    pub id: String,
    pub name: String,
}

#[async_trait]
impl GetAll for Fleet {
    async fn all(connection: &sqlx::SqlitePool) -> Result<Vec<Self>> {
        let mut conn = connection.acquire().await?;
        let r = sqlx::query_as!(Self, "select * from fleets")
            .fetch_all(&mut conn)
            .await?;
        Ok(r)
    }
}
